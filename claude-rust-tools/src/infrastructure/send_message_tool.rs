use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

/// Tool to send a message to an agent or team.
///
/// TODO: Phase 5 — integrate with real message routing.
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
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'to' field".into()))?;

        let message = input
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'message' field".into()))?;

        // TODO: Phase 5 — replace stub with real message routing
        tracing::info!(to, message_len = message.len(), "sending message (stub)");

        Ok(format!("Message delivered to '{to}'. Length: {} chars.", message.len()))
    }
}
