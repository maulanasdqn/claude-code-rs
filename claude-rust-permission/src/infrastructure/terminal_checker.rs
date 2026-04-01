use std::io::Write;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionChecker, PermissionDecision};
use crossterm::{event, terminal};
use serde_json::Value;

use crate::application::{format_permission_prompt, permission_title};

pub struct InteractivePermissionChecker {
    paused: Arc<AtomicBool>,
}

impl InteractivePermissionChecker {
    pub fn new() -> Self {
        Self { paused: Arc::new(AtomicBool::new(false)) }
    }

    pub fn with_flag(paused: Arc<AtomicBool>) -> Self {
        Self { paused }
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
        let detail = format_permission_prompt(tool_name, input);
        let title = permission_title(tool_name);
        let paused = self.paused.clone();

        let answer = tokio::task::spawn_blocking(move || -> AppResult<bool> {
            paused.store(true, Ordering::Relaxed);
            let result = prompt_raw(&title, &detail);
            paused.store(false, Ordering::Relaxed);
            result
        })
        .await
        .map_err(|e| AppError::Tool(format!("permission task failed: {e}")))??;

        if answer {
            Ok(PermissionDecision::Allow)
        } else {
            Ok(PermissionDecision::Deny(format!("user denied permission for \"{tool_name}\"")))
        }
    }
}

fn prompt_raw(title: &str, detail: &str) -> AppResult<bool> {
    let w = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80).saturating_sub(4).max(24);
    let inner = w.saturating_sub(2);
    let hint = " [Y]es  [N]o ";
    let top_label = format!("─ {title} ");
    let top_fill = inner.saturating_sub(top_label.len());
    let bot_fill = inner.saturating_sub(hint.len());

    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr);
    let _ = writeln!(stderr, "  \x1b[33m\x1b[1m╭{top_label}{}\x1b[0m", "─".repeat(top_fill));
    let detail_max = inner.saturating_sub(4);
    let detail_display = if detail.len() > detail_max { &detail[..detail_max] } else { detail };
    let pad = detail_max.saturating_sub(detail_display.len());
    let _ = writeln!(stderr, "  \x1b[33m│\x1b[0m  {detail_display}{}  \x1b[33m│\x1b[0m", " ".repeat(pad));
    let _ = writeln!(stderr, "  \x1b[33m\x1b[1m╰{}\x1b[2m{hint}\x1b[0m\x1b[33m\x1b[1m╯\x1b[0m", "─".repeat(bot_fill));
    let _ = stderr.flush();

    terminal::enable_raw_mode().map_err(|e| AppError::Tool(e.to_string()))?;
    let result = read_keypress();
    terminal::disable_raw_mode().ok();

    let lines_to_clear = 4u16;
    let _ = write!(std::io::stderr(), "\x1b[{}A\r\x1b[J", lines_to_clear);
    let _ = std::io::stderr().flush();

    result
}

fn read_keypress() -> AppResult<bool> {
    loop {
        match event::read().map_err(|e| AppError::Tool(e.to_string()))? {
            event::Event::Key(k) => match k.code {
                event::KeyCode::Char('y') | event::KeyCode::Char('Y') | event::KeyCode::Enter => return Ok(true),
                event::KeyCode::Char('n') | event::KeyCode::Char('N') | event::KeyCode::Esc => return Ok(false),
                _ => {}
            },
            _ => {}
        }
    }
}
