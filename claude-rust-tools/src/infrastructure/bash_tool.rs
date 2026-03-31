use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

pub struct BashTool;

#[async_trait::async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command and return its output."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'command' field".into()))?;

        tracing::info!(command, "executing bash");

        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| AppError::Tool(format!("failed to spawn bash: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
        }
        if result.is_empty() {
            result.push_str("(no output)");
        }

        if result.len() > 100_000 {
            result.truncate(100_000);
            result.push_str("\n... (truncated)");
        }

        Ok(result)
    }
}
