use stynx_code_errors::AppResult;
use stynx_code_types::{InterruptBehavior, PermissionLevel, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

/// Tool to execute code snippets in a REPL subprocess (Python or Node.js).
pub struct REPLTool;

impl REPLTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for REPLTool {
    fn name(&self) -> &str {
        "repl"
    }

    fn description(&self) -> &str {
        "Execute code in a Python or Node.js subprocess and return the output."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "description": "Programming language: \"python\" or \"node\"",
                    "enum": ["python", "node"]
                },
                "code": {
                    "type": "string",
                    "description": "The code to execute"
                }
            },
            "required": ["language", "code"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn interrupt_behavior(&self) -> InterruptBehavior {
        InterruptBehavior::Cancel
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let language = input
            .get("language")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'language' field".into()))?;

        let code = input
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'code' field".into()))?;

        let (program, flag) = match language {
            "python" => ("python3", "-c"),
            "node" => ("node", "-e"),
            other => return Err(stynx_code_errors::AppError::Tool(
                format!("unsupported language: {other}. Use 'python' or 'node'.")
            )),
        };

        tracing::info!(language, code_len = code.len(), "executing REPL");

        let output = Command::new(program)
            .arg(flag)
            .arg(code)
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
