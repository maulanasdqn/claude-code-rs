use std::collections::HashSet;
use std::io::Write;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionChecker, PermissionDecision};
use crossterm::{event, terminal};
use serde_json::Value;

use crate::application::{format_permission_prompt, permission_title};

pub struct InteractivePermissionChecker {
    paused: Arc<AtomicBool>,
    session_allowed: Arc<Mutex<HashSet<String>>>,
}

impl InteractivePermissionChecker {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            session_allowed: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn with_flag(paused: Arc<AtomicBool>) -> Self {
        Self { paused, session_allowed: Arc::new(Mutex::new(HashSet::new())) }
    }

    pub fn pause_flag(&self) -> Arc<AtomicBool> {
        self.paused.clone()
    }
}

impl Default for InteractivePermissionChecker {
    fn default() -> Self { Self::new() }
}

#[async_trait::async_trait]
impl PermissionChecker for InteractivePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        if self.session_allowed.lock().unwrap().contains(tool_name) {
            return Ok(PermissionDecision::Allow);
        }

        let detail = format_permission_prompt(tool_name, input);
        let title = permission_title(tool_name);
        let tool_name = tool_name.to_string();
        let tool_name_for_session = tool_name.clone();
        let tool_name_for_deny = tool_name.clone();
        let paused = self.paused.clone();
        let session_allowed = self.session_allowed.clone();

        let decision = tokio::task::spawn_blocking(move || -> AppResult<SelectResult> {
            paused.store(true, Ordering::Relaxed);
            let result = prompt_select(&title, &detail, &tool_name);
            paused.store(false, Ordering::Relaxed);
            result
        })
        .await
        .map_err(|e| AppError::Tool(format!("permission task failed: {e}")))??;

        match decision {
            SelectResult::AllowOnce => Ok(PermissionDecision::Allow),
            SelectResult::AllowAlways => {
                session_allowed.lock().unwrap().insert(tool_name_for_session);
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

fn prompt_select(title: &str, detail: &str, tool_name: &str) -> AppResult<SelectResult> {
    let w = terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
        .saturating_sub(4)
        .max(24);
    let inner = w.saturating_sub(2);
    let detail_max = inner.saturating_sub(4);
    let detail_display = if detail.len() > detail_max { &detail[..detail_max] } else { detail };
    let pad = detail_max.saturating_sub(detail_display.len());
    let top_label = format!("─ {title} ");
    let top_fill = inner.saturating_sub(top_label.len());

    let always_label = format!("Yes, and don't ask again for {tool_name}");
    let options: Vec<(&str, SelectResult)> = vec![
        ("Yes", SelectResult::AllowOnce),
        (&always_label, SelectResult::AllowAlways),
        ("No", SelectResult::Deny),
    ];

    let mut selected = 0usize;
    let n = options.len();

    let mut out = std::io::stdout();

    let draw = |out: &mut dyn Write, sel: usize| -> std::io::Result<()> {
        writeln!(out, "\n  \x1b[33m\x1b[1m╭{top_label}{}\x1b[0m", "─".repeat(top_fill))?;
        writeln!(out, "  \x1b[33m│\x1b[0m  {detail_display}{}  \x1b[33m│\x1b[0m", " ".repeat(pad))?;
        writeln!(out, "  \x1b[33m\x1b[1m╰{}\x1b[0m", "─".repeat(inner))?;
        for (i, (label, _)) in options.iter().enumerate() {
            if i == sel {
                writeln!(out, "  \x1b[33m\x1b[1m❯\x1b[0m \x1b[1m{label}\x1b[0m")?;
            } else {
                writeln!(out, "    \x1b[2m{label}\x1b[0m")?;
            }
        }
        out.flush()
    };

    let clear = |out: &mut dyn Write| -> std::io::Result<()> {
        let lines = (3 + n + 1) as u16;
        write!(out, "\x1b[{lines}A\r\x1b[J")?;
        out.flush()
    };

    draw(&mut out, selected).ok();

    terminal::enable_raw_mode().map_err(|e| AppError::Tool(e.to_string()))?;

    let result = loop {
        if !event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            continue;
        }
        use event::{Event, KeyCode, KeyModifiers};
        match event::read().map_err(|e| AppError::Tool(e.to_string()))? {
            Event::Key(k) => match (k.code, k.modifiers) {
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
            },
            _ => {}
        }
    };

    terminal::disable_raw_mode().ok();
    clear(&mut out).ok();

    result
}
