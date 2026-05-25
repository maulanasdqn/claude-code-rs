use std::sync::Arc;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskGetTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskGetTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskGetTool {
    fn name(&self) -> &str {
        "task_get"
    }

    fn description(&self) -> &str {
        "Get detailed information about a specific task by its ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The ID of the task to retrieve"
                }
            },
            "required": ["task_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task_id = input
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'task_id' field".into()))?;

        let task = self.manager.get(task_id).await?;

        serde_json::to_string_pretty(&task)
            .map_err(|e| AppError::Tool(format!("failed to serialize task: {e}")))
    }
}
