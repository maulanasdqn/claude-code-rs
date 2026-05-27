use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct SendMessageTool;

impl SendMessageTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for SendMessageTool {
    fn name(&self) -> &str {
        "send_message"
    }

    fn description(&self) -> &str {
        "Send a message to a named agent or team."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Name of the agent or team to send the message to"
                },
                "message": {
                    "type": "string",
                    "description": "The message content to send"
                }
            },
            "required": ["to", "message"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let to = input
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'to' field".into()))?;

        let message = input
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'message' field".into()))?;

        tracing::info!(to, message_len = message.len(), "sending message (stub)");

        Ok(format!("Message delivered to '{to}'. Length: {} chars.", message.len()))
    }
}
