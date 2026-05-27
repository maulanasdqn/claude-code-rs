use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct CronDeleteTool;

impl CronDeleteTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for CronDeleteTool {
    fn name(&self) -> &str {
        "cron_delete"
    }

    fn description(&self) -> &str {
        "Delete an existing cron job by its ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cron_id": {
                    "type": "string",
                    "description": "The ID of the cron job to delete"
                }
            },
            "required": ["cron_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let cron_id = input
            .get("cron_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'cron_id' field".into()))?;

        tracing::info!(cron_id, "deleting cron job (stub)");

        Ok(format!("Cron job '{cron_id}' deleted successfully."))
    }
}
