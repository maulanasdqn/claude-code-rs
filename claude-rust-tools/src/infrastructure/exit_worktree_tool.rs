use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

pub struct ExitWorktreeTool;

impl ExitWorktreeTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for ExitWorktreeTool {
    fn name(&self) -> &str {
        "exit_worktree"
    }

    fn description(&self) -> &str {
        "Exit and optionally remove a git worktree."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path of the worktree to exit/remove"
                },
                "remove": {
                    "type": "boolean",
                    "description": "Whether to remove the worktree (default false)"
                }
            },
            "required": ["path"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'path' field".into()))?;

        let remove = input
            .get("remove")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        tracing::info!(path, remove, "exiting worktree");

        if remove {
            let output = Command::new("git")
                .args(["worktree", "remove", path])
                .output()
                .await
                .map_err(|e| AppError::Tool(format!("failed to run git: {e}")))?;

            if output.status.success() {
                Ok(format!("Removed worktree at '{path}'."))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(AppError::Tool(format!("git worktree remove failed: {stderr}")))
            }
        } else {
            Ok(format!("Exited worktree at '{path}'. Use remove=true to delete it."))
        }
    }
}
