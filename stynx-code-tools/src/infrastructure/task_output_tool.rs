use std::sync::Arc;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskOutputTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskOutputTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskOutputTool {
    fn name(&self) -> &str {
        "task_output"
    }

    fn description(&self) -> &str {
        "Get the output produced by a task. Returns the captured stdout/stderr from the task's subprocess."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The ID of the task whose output to retrieve"
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

        let output = self.manager.output(task_id).await?;

        if output.is_empty() {
            Ok("No output available.".into())
        } else {
            Ok(output)
        }
    }
}
