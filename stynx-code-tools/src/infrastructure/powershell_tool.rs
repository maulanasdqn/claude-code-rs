use stynx_code_errors::AppResult;
use stynx_code_types::{InterruptBehavior, PermissionLevel, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

/// Tool to execute PowerShell commands.
pub struct PowerShellTool;

impl PowerShellTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for PowerShellTool {
    fn name(&self) -> &str {
        "powershell"
    }

    fn description(&self) -> &str {
        "Execute a PowerShell command and return its output."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The PowerShell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Optional timeout in seconds"
                }
            },
            "required": ["command"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn interrupt_behavior(&self) -> InterruptBehavior {
        InterruptBehavior::Cancel
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'command' field".into()))?;

        let _timeout = input
            .get("timeout")
            .and_then(|v| v.as_u64());

        tracing::info!(command, "executing PowerShell");

        // Use pwsh (cross-platform) or powershell.exe on Windows
        let program = if cfg!(target_os = "windows") {
            "powershell.exe"
        } else {
            "pwsh"
        };

        let output = Command::new(program)
            .arg("-Command")
            .arg(command)
            .output()
            .await
            .map_err(|e| stynx_code_errors::AppError::Tool(
                format!("failed to spawn {program}: {e}")
            ))?;

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
