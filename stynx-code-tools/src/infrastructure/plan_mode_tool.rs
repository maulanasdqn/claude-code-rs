use std::io::Write;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct EnterPlanModeTool;

#[async_trait::async_trait]
impl Tool for EnterPlanModeTool {
    fn name(&self) -> &str { "enter_plan_mode" }
    fn description(&self) -> &str { "Enter plan mode." }
    fn input_schema(&self) -> Value { json!({"type":"object","properties":{},"required":[]}) }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, _input: Value) -> AppResult<String> {
        Ok("Plan mode activated.".into())
    }
}

pub struct ExitPlanModeTool {
    paused: Arc<AtomicBool>,
}

impl ExitPlanModeTool {
    pub fn new(paused: Arc<AtomicBool>) -> Self {
        Self { paused }
    }
}

#[async_trait::async_trait]
impl Tool for ExitPlanModeTool {
    fn name(&self) -> &str { "exit_plan_mode" }

    fn description(&self) -> &str {
        "Signal that you have finished planning. Call this after presenting your complete plan as text to the user. The user will review and approve your plan before any changes are made."
    }

    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{},"required":[]})
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn requires_user_interaction(&self) -> bool { true }

    async fn execute(&self, _input: Value) -> AppResult<String> {
        let paused = self.paused.clone();
        tokio::task::spawn_blocking(move || {
            paused.store(true, Ordering::Relaxed);
            let result = confirm_exit_plan();
            paused.store(false, Ordering::Relaxed);
            result
        })
        .await
        .map_err(|e| AppError::Tool(format!("exit plan task failed: {e}")))??;

        Ok("User approved the plan. Proceeding with implementation.".into())
    }
}

fn confirm_exit_plan() -> AppResult<String> {
    use crossterm::{
        event::{self, KeyCode, KeyModifiers},
        terminal,
    };

    let mut out = std::io::stdout();
    let _ = writeln!(out, "\n  \x1b[35m\x1b[1m◆  Exit plan mode?\x1b[0m  \x1b[2m[y/n]\x1b[0m");
    let _ = out.flush();

    terminal::enable_raw_mode().map_err(|e| AppError::Tool(e.to_string()))?;

    let result = loop {
        if !event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        if let event::Event::Key(k) = event::read().map_err(|e| AppError::Tool(e.to_string()))? { match (k.code, k.modifiers) {
            (KeyCode::Char('y'), _) | (KeyCode::Char('Y'), _) | (KeyCode::Enter, _) => {
                let _ = writeln!(out, "\r  \x1b[35m\x1b[1m◆  Plan approved\x1b[0m\n");
                let _ = out.flush();
                break Ok(String::new());
            }
            (KeyCode::Char('n'), _) | (KeyCode::Char('N'), _) | (KeyCode::Esc, _) => {
                let _ = writeln!(out, "\r  \x1b[2m◆  Plan rejected — continue refining\x1b[0m\n");
                let _ = out.flush();
                break Err(AppError::Tool(
                    "Plan rejected by user. Refine your plan and call exit_plan_mode again when ready.".into(),
                ));
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                break Err(AppError::Interrupted);
            }
            _ => {}
        } }
    };

    terminal::disable_raw_mode().ok();
    result
}
