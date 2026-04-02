use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct FileEditTool;

#[async_trait::async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str {
        "file_edit"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing an exact string match. The old_string must appear exactly once."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact string to find and replace (must be unique in the file)"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement string"
                }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn get_path(&self, input: &Value) -> Option<String> {
        input.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'file_path' field".into()))?;

        let old_string = input
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'old_string' field".into()))?;

        let new_string = input
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'new_string' field".into()))?;

        tracing::info!(path, "editing file");

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AppError::Tool(format!("cannot read '{path}': {e}")))?;

        let match_count = content.matches(old_string).count();
        if match_count == 0 {
            return Err(AppError::Tool(format!(
                "old_string not found in '{path}'"
            )));
        }
        if match_count > 1 {
            return Err(AppError::Tool(format!(
                "old_string found {match_count} times in '{path}' (must be unique)"
            )));
        }

        let new_content = content.replacen(old_string, new_string, 1);
        tokio::fs::write(path, &new_content)
            .await
            .map_err(|e| AppError::Tool(format!("cannot write '{path}': {e}")))?;

        Ok(format!("Edited {path}"))
    }
}
