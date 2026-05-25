use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct SleepTool;

impl SleepTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for SleepTool {
    fn name(&self) -> &str {
        "sleep"
    }

    fn description(&self) -> &str {
        "Sleep for a specified number of seconds (max 300)."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "seconds": {
                    "type": "integer",
                    "description": "Number of seconds to sleep (max 300)"
                }
            },
            "required": ["seconds"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let seconds = input
            .get("seconds")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| AppError::Tool("missing 'seconds' field".into()))?;

        let seconds = seconds.min(300);

        tracing::info!(seconds, "sleeping");

        let start = std::time::Instant::now();
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        let elapsed = start.elapsed();

        Ok(format!("Slept for {:.2} seconds.", elapsed.as_secs_f64()))
    }
}
