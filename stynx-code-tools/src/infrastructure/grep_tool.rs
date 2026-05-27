use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};
use tokio::process::Command;

pub struct GrepTool;

#[async_trait::async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regex. Uses ripgrep (rg) if available, falls back to grep."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in (defaults to current working directory)"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g. \"*.rs\", \"*.{ts,tsx}\")"
                }
            },
            "required": ["pattern"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: true, is_read: false, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'pattern' field".into()))?;

        let search_path = input
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let file_glob = input.get("glob").and_then(|v| v.as_str());

        tracing::info!(pattern, search_path, "grepping");

        let output = match try_ripgrep(pattern, search_path, file_glob).await {
            Ok(out) => out,
            Err(_) => try_grep(pattern, search_path).await?,
        };

        if output.is_empty() {
            return Ok("No matches found.".into());
        }

        let mut result = output;
        if result.len() > 100_000 {
            result.truncate(100_000);
            result.push_str("\n... (truncated)");
        }

        Ok(result)
    }
}

async fn try_ripgrep(
    pattern: &str,
    path: &str,
    file_glob: Option<&str>,
) -> AppResult<String> {
    let mut cmd = Command::new("rg");
    cmd.arg("-n").arg("--no-heading").arg(pattern);

    if let Some(g) = file_glob {
        cmd.arg("--glob").arg(g);
    }

    cmd.arg(path);

    let output = cmd
        .output()
        .await
        .map_err(|e| AppError::Tool(format!("rg not available: {e}")))?;

    if output.status.code() == Some(2) {
        return Err(AppError::Tool("ripgrep error".into()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn try_grep(pattern: &str, path: &str) -> AppResult<String> {
    let output = Command::new("grep")
        .arg("-rn")
        .arg(pattern)
        .arg(path)
        .output()
        .await
        .map_err(|e| AppError::Tool(format!("grep failed: {e}")))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
