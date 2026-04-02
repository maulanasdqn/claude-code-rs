use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

/// Tool to create a cron job.
///
/// TODO: Phase 4 — integrate with real cron scheduler.
pub struct CronCreateTool;

impl CronCreateTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for CronCreateTool {
    fn name(&self) -> &str {
        "cron_create"
    }

    fn description(&self) -> &str {
        "Create a new cron job with a schedule expression and command."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "schedule": {
                    "type": "string",
                    "description": "Cron expression (e.g. \"0 */5 * * *\")"
                },
                "command": {
                    "type": "string",
                    "description": "Command to execute on the schedule"
                },
                "name": {
                    "type": "string",
                    "description": "Optional human-readable name for the cron job"
                }
            },
            "required": ["schedule", "command"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let schedule = input
            .get("schedule")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'schedule' field".into()))?;

        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'command' field".into()))?;

        let name = input.get("name").and_then(|v| v.as_str());

        // TODO: Phase 4 — replace stub with real cron scheduler integration
        let cron_id = format!("cron-{:08x}", rand_id());

        tracing::info!(schedule, command, ?name, cron_id, "creating cron job (stub)");

        Ok(format!(
            "Cron job created.\n  id: {cron_id}\n  schedule: {schedule}\n  command: {command}{}",
            name.map(|n| format!("\n  name: {n}")).unwrap_or_default()
        ))
    }
}

fn rand_id() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    h.finish() as u32
}
