use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

/// Tool that returns content as-is for structured output.
pub struct BriefTool;

impl BriefTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for BriefTool {
    fn name(&self) -> &str {
        "brief"
    }

    fn description(&self) -> &str {
        "Return content as structured output, optionally with a title."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The content to output"
                },
                "title": {
                    "type": "string",
                    "description": "Optional title for the output"
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
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'content' field".into()))?;

        let title = input.get("title").and_then(|v| v.as_str());

        tracing::info!(title, content_len = content.len(), "brief output");

        match title {
            Some(t) => Ok(format!("## {t}\n\n{content}")),
            None => Ok(content.to_string()),
        }
    }
}
