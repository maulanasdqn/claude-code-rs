use std::collections::HashSet;

fn input_signature(tool_name: &str, input: &Value) -> String {
    match tool_name {
        "bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let first = cmd.split_whitespace().next().unwrap_or("*");
            format!("bash|{first}")
        }
        "file_edit" | "file_write" | "read" => {
            let path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("*");
            format!("{tool_name}|{path}")
        }
        other => format!("{other}|*"),
    }
}

fn signature_label(tool_name: &str, input: &Value) -> String {
    match tool_name {
        "bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let first = cmd.split_whitespace().next().unwrap_or("");
            if first.is_empty() {
                format!("`bash` (any command)")
            } else {
                format!("`bash {first} ...` commands")
            }
        }
        "file_edit" | "file_write" | "read" => {
            let path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            if path.is_empty() {
                format!("`{tool_name}` (any path)")
            } else {
                format!("`{tool_name}` on `{path}`")
            }
        }
        other => format!("`{other}`"),
    }
}
use std::io::Write;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionChecker, PermissionDecision};
use crossterm::{event, terminal};
use serde_json::Value;

use crate::application::{format_permission_prompt, permission_title};
use crate::infrastructure::prompt_bridge::{OptionalBridge, PromptChoice, SharedBridge};

pub struct InteractivePermissionChecker {
    paused: Arc<AtomicBool>,
    session_allowed: Arc<Mutex<HashSet<String>>>,
    prompt_lock: Arc<tokio::sync::Mutex<()>>,
    bridge: SharedBridge,
}

impl InteractivePermissionChecker {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            session_allowed: Arc::new(Mutex::new(HashSet::new())),
            prompt_lock: Arc::new(tokio::sync::Mutex::new(())),
            bridge: Arc::new(OptionalBridge::new()),
        }
    }

    pub fn with_flag(paused: Arc<AtomicBool>) -> Self {
        Self {
            paused,
            session_allowed: Arc::new(Mutex::new(HashSet::new())),
            prompt_lock: Arc::new(tokio::sync::Mutex::new(())),
            bridge: Arc::new(OptionalBridge::new()),
        }
    }

    pub fn pause_flag(&self) -> Arc<AtomicBool> {
        self.paused.clone()
    }

    pub fn bridge_handle(&self) -> SharedBridge {
        self.bridge.clone()
    }
}

impl Default for InteractivePermissionChecker {
    fn default() -> Self { Self::new() }
}

#[async_trait::async_trait]
impl PermissionChecker for InteractivePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        let sig = input_signature(tool_name, input);
        if self.session_allowed.lock().unwrap().contains(&sig) {
            return Ok(PermissionDecision::Allow);
        }

        let detail = format_permission_prompt(tool_name, input);
        let title = permission_title(tool_name);
        let allow_scope = signature_label(tool_name, input);
        let tool_name = tool_name.to_string();
        let session_sig = sig;
        let session_sig_blocking = session_sig.clone();
        let tool_name_for_deny = tool_name.clone();

        if let Some(bridge) = self.bridge.get() {
            let _guard = self.prompt_lock.lock().await;
            let scoped_detail = format!("{detail}\n\n(\"don't ask again\" only covers {allow_scope})");
            let choice = bridge.request(title, scoped_detail).await;
            return Ok(match choice {
                Some(PromptChoice::AllowOnce) => PermissionDecision::Allow,
                Some(PromptChoice::AllowAlways) => {
                    self.session_allowed.lock().unwrap().insert(session_sig);
                    PermissionDecision::Allow
                }
                Some(PromptChoice::Deny) | None => PermissionDecision::Deny(format!(
                    "user denied permission for \"{tool_name_for_deny}\""
                )),
            });
        }

        let paused = self.paused.clone();
        let session_allowed = self.session_allowed.clone();

        let _guard = self.prompt_lock.lock().await;

        let allow_scope_for_prompt = allow_scope.clone();
        let decision = tokio::task::spawn_blocking(move || -> AppResult<SelectResult> {
            paused.store(true, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(80));
            let result = prompt_select(&title, &detail, &tool_name, &allow_scope_for_prompt);
            paused.store(false, Ordering::Relaxed);
            std::thread::sleep(std::time::Duration::from_millis(50));
            result
        })
        .await
        .map_err(|e| AppError::Tool(format!("permission task failed: {e}")))??;

        match decision {
            SelectResult::AllowOnce => Ok(PermissionDecision::Allow),
            SelectResult::AllowAlways => {
                session_allowed.lock().unwrap().insert(session_sig_blocking);
                Ok(PermissionDecision::Allow)
            }
            SelectResult::Deny => Ok(PermissionDecision::Deny(
                format!("user denied permission for \"{tool_name_for_deny}\""),
            )),
        }
    }
}

#[derive(Clone, Copy)]
enum SelectResult {
    AllowOnce,
    AllowAlways,
    Deny,
}

fn prompt_select(title: &str, detail: &str, tool_name: &str, allow_scope: &str) -> AppResult<SelectResult> {
    let w = terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
        .saturating_sub(4)
        .max(24);
    let inner = w;

    let detail_lines: Vec<String> = {
        let mut lines = Vec::new();
        let mut remaining = detail;
        while remaining.len() > inner {

            let break_at = remaining[..inner].rfind(' ').unwrap_or(inner);
            lines.push(remaining[..break_at].to_string());
            remaining = remaining[break_at..].trim_start();
        }
        if !remaining.is_empty() {
            lines.push(remaining.to_string());
        }
        if lines.is_empty() { lines.push(String::new()); }
        lines
    };
    let detail_line_count = detail_lines.len();

    let title_prefix = format!("─── {title} ");
    let title_fill = inner.saturating_sub(title_prefix.len());

    let _ = tool_name;
    let always_label = format!("Yes, and don't ask again for {allow_scope}");
    let options: Vec<(&str, SelectResult)> = vec![
        ("Yes", SelectResult::AllowOnce),
        (&always_label, SelectResult::AllowAlways),
        ("No", SelectResult::Deny),
    ];

    let mut selected = 0usize;
    let n = options.len();

    let mut out = std::io::stdout();

    let draw = |out: &mut dyn Write, sel: usize| -> std::io::Result<()> {

        write!(out, "\r\n  \x1b[33m\x1b[1m{title_prefix}{}\x1b[0m\r\n", "─".repeat(title_fill))?;

        for line in &detail_lines {
            write!(out, "  \x1b[2m{line}\x1b[0m\r\n")?;
        }

        write!(out, "  \x1b[2m{}\x1b[0m\r\n", "─".repeat(inner))?;

        write!(out, "\r\n")?;

        for (i, (label, _)) in options.iter().enumerate() {
            if i == sel {
                write!(out, "  \x1b[33m\x1b[1m❯\x1b[0m \x1b[1m{label}\x1b[0m\r\n")?;
            } else {
                write!(out, "    \x1b[2m{label}\x1b[0m\r\n")?;
            }
        }
        out.flush()
    };

    let total_lines = (4 + detail_line_count + n) as u16;

    let clear = |out: &mut dyn Write| -> std::io::Result<()> {
        write!(out, "\x1b[{total_lines}A\r\x1b[J")?;
        out.flush()
    };

    draw(&mut out, selected).ok();

    terminal::enable_raw_mode().map_err(|e| AppError::Tool(e.to_string()))?;

    let result = loop {
        if !event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            continue;
        }
        use event::{Event, KeyCode, KeyModifiers};
        if let Event::Key(k) = event::read().map_err(|e| AppError::Tool(e.to_string()))? { match (k.code, k.modifiers) {
            (KeyCode::Up, _) => {
                selected = selected.checked_sub(1).unwrap_or(n - 1);
                clear(&mut out).ok();
                draw(&mut out, selected).ok();
            }
            (KeyCode::Down, _) => {
                selected = (selected + 1) % n;
                clear(&mut out).ok();
                draw(&mut out, selected).ok();
            }
            (KeyCode::Enter, _) => break Ok(options[selected].1),
            (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), KeyModifiers::NONE) => {
                break Ok(SelectResult::AllowOnce);
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('A'), KeyModifiers::NONE) => {
                break Ok(SelectResult::AllowAlways);
            }
            (KeyCode::Char('n'), KeyModifiers::NONE) | (KeyCode::Char('N'), KeyModifiers::NONE)
            | (KeyCode::Esc, _) => {
                break Ok(SelectResult::Deny);
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                break Err(AppError::Interrupted);
            }
            _ => {}
        } }
    };

    terminal::disable_raw_mode().ok();
    clear(&mut out).ok();

    result
}
