use std::io::Write;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionChecker, PermissionDecision};
use serde_json::Value;

use crate::application::format_permission_prompt;

pub struct InteractivePermissionChecker;

impl InteractivePermissionChecker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InteractivePermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PermissionChecker for InteractivePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        let prompt = format_permission_prompt(tool_name, input);

        let answer = tokio::task::spawn_blocking(move || -> AppResult<String> {
            let mut stderr = std::io::stderr().lock();
            stderr
                .write_all(prompt.as_bytes())
                .map_err(|e| AppError::Tool(format!("failed to write permission prompt: {e}")))?;
            stderr
                .flush()
                .map_err(|e| AppError::Tool(format!("failed to flush stderr: {e}")))?;

            let mut line = String::new();
            std::io::stdin()
                .read_line(&mut line)
                .map_err(|e| AppError::Tool(format!("failed to read permission response: {e}")))?;
            Ok(line)
        })
        .await
        .map_err(|e| AppError::Tool(format!("permission prompt task failed: {e}")))??;

        let trimmed = answer.trim().to_lowercase();
        if trimmed.starts_with('y') {
            Ok(PermissionDecision::Allow)
        } else {
            Ok(PermissionDecision::Deny(format!(
                "user denied permission for tool \"{tool_name}\""
            )))
        }
    }
}
