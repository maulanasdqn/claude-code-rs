use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct ReadTool;

#[async_trait::async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns numbered lines."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-based)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read"
                }
            },
            "required": ["file_path"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'file_path' field".into()))?;

        let offset = input
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .max(1) as usize;

        let limit = input
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(2000) as usize;

        tracing::info!(path, offset, limit, "reading file");

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AppError::Tool(format!("cannot read '{path}': {e}")))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = (offset - 1).min(lines.len());
        let end = (start + limit).min(lines.len());

        let mut result = String::new();
        for (i, line) in lines[start..end].iter().enumerate() {
            let line_num = start + i + 1;
            result.push_str(&format!("{line_num:>6}\t{line}\n"));
        }

        if result.is_empty() {
            result.push_str("(empty file)");
        }

        Ok(result)
    }
}
