use std::sync::Arc;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskStopTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskStopTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskStopTool {
    fn name(&self) -> &str {
        "task_stop"
    }

    fn description(&self) -> &str {
        "Stop and delete a running task by its ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The ID of the task to stop"
                }
            },
            "required": ["task_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task_id = input
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'task_id' field".into()))?;

        self.manager.stop(task_id).await?;

        Ok(format!("Task {task_id} stopped"))
    }
}
