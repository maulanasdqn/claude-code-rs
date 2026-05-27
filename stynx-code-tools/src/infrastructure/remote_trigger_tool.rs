use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct RemoteTriggerTool;

impl RemoteTriggerTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for RemoteTriggerTool {
    fn name(&self) -> &str {
        "remote_trigger"
    }

    fn description(&self) -> &str {
        "Fire a remote trigger by ID, optionally with a JSON payload."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "trigger_id": {
                    "type": "string",
                    "description": "The ID of the remote trigger to fire"
                },
                "payload": {
                    "description": "Optional JSON payload to send with the trigger"
                }
            },
            "required": ["trigger_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let trigger_id = input
            .get("trigger_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'trigger_id' field".into()))?;

        let payload = input.get("payload");

        tracing::info!(trigger_id, ?payload, "firing remote trigger (stub)");

        Ok(format!("Trigger '{trigger_id}' acknowledged. (stub)"))
    }
}
