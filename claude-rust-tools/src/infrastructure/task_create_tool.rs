use std::sync::Arc;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskCreateTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskCreateTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskCreateTool {
    fn name(&self) -> &str {
        "task_create"
    }

    fn description(&self) -> &str {
        "Create a new task with a subject and description. Returns the created task info including its generated ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "subject": {
                    "type": "string",
                    "description": "Short subject line for the task"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of the task"
                },
                "metadata": {
                    "type": "object",
                    "description": "Optional metadata to attach to the task"
                }
            },
            "required": ["subject", "description"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let subject = input
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'subject' field".into()))?;

        let description = input
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'description' field".into()))?;

        let mut task = self.manager.create(subject, description).await?;

        if let Some(metadata) = input.get("metadata") {
            task.metadata = metadata.clone();
        }

        serde_json::to_string_pretty(&task)
            .map_err(|e| AppError::Tool(format!("failed to serialize task: {e}")))
    }
}
