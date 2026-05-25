use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

pub struct EnterWorktreeTool;

impl EnterWorktreeTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for EnterWorktreeTool {
    fn name(&self) -> &str {
        "enter_worktree"
    }

    fn description(&self) -> &str {
        "Create a git worktree for the specified branch."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "branch": {
                    "type": "string",
                    "description": "Branch name to create the worktree for"
                },
                "path": {
                    "type": "string",
                    "description": "Path for the worktree directory (defaults to ../branch-name)"
                }
            },
            "required": ["branch"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn is_destructive(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let branch = input
            .get("branch")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'branch' field".into()))?;

        let default_path = format!("../{branch}");
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(&default_path);

        tracing::info!(branch, path, "creating worktree");

        // Try creating a new branch first; if that fails, use existing branch
        let output = Command::new("git")
            .args(["worktree", "add", path, "-b", branch])
            .output()
            .await
            .map_err(|e| AppError::Tool(format!("failed to run git: {e}")))?;

        if output.status.success() {
            return Ok(format!("Created worktree at '{path}' on new branch '{branch}'."));
        }

        // Fall back to existing branch
        let output = Command::new("git")
            .args(["worktree", "add", path, branch])
            .output()
            .await
            .map_err(|e| AppError::Tool(format!("failed to run git: {e}")))?;

        if output.status.success() {
            Ok(format!("Created worktree at '{path}' on existing branch '{branch}'."))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Tool(format!("git worktree add failed: {stderr}")))
        }
    }
}
