use std::sync::Arc;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskUpdateTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskUpdateTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskUpdateTool {
    fn name(&self) -> &str {
        "task_update"
    }

    fn description(&self) -> &str {
        "Update an existing task. Can change status, subject, description, or owner."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The ID of the task to update"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "deleted"],
                    "description": "New status for the task"
                },
                "subject": {
                    "type": "string",
                    "description": "New subject line for the task"
                },
                "description": {
                    "type": "string",
                    "description": "New description for the task"
                },
                "owner": {
                    "type": "string",
                    "description": "New owner for the task"
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

        let mut updates = serde_json::Map::new();
        if let Some(v) = input.get("status") {
            updates.insert("status".into(), v.clone());
        }
        if let Some(v) = input.get("subject") {
            updates.insert("subject".into(), v.clone());
        }
        if let Some(v) = input.get("description") {
            updates.insert("description".into(), v.clone());
        }
        if let Some(v) = input.get("owner") {
            updates.insert("owner".into(), v.clone());
        }

        let task = self.manager.update(task_id, Value::Object(updates)).await?;

        serde_json::to_string_pretty(&task)
            .map_err(|e| AppError::Tool(format!("failed to serialize task: {e}")))
    }
}
