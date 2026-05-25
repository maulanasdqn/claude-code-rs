use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct FileWriteTool;

#[async_trait::async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file, creating parent directories as needed."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn is_destructive(&self, _input: &Value) -> bool { true }

    fn get_path(&self, input: &Value) -> Option<String> {
        input.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'file_path' field".into()))?;

        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'content' field".into()))?;

        tracing::info!(path, "writing file");

        let file_path = std::path::Path::new(path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Tool(format!("cannot create directories for '{path}': {e}")))?;
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| AppError::Tool(format!("cannot write '{path}': {e}")))?;

        let line_count = content.lines().count();
        Ok(format!("Wrote {line_count} lines to {path}"))
    }
}
