use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct SyntheticOutputTool;

impl SyntheticOutputTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for SyntheticOutputTool {
    fn name(&self) -> &str {
        "synthetic_output"
    }

    fn description(&self) -> &str {
        "Return content as structured output in a specified format (text, json, or markdown)."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The content to output"
                },
                "format": {
                    "type": "string",
                    "description": "Output format: \"text\", \"json\", or \"markdown\"",
                    "enum": ["text", "json", "markdown"]
                }
            },
            "required": ["content"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'content' field".into()))?;

        let format = input
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        tracing::info!(format, content_len = content.len(), "synthetic output");

        match format {
            "json" => {

                if serde_json::from_str::<Value>(content).is_ok() {
                    Ok(content.to_string())
                } else {
                    Ok(json!({ "content": content }).to_string())
                }
            }
            "markdown" | "text" => Ok(content.to_string()),
            other => Err(stynx_code_errors::AppError::Tool(
                format!("unsupported format: {other}. Use 'text', 'json', or 'markdown'.")
            )),
        }
    }
}
