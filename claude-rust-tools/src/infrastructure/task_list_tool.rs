use std::sync::Arc;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

use super::task_manager::TaskManager;

pub struct TaskListTool {
    manager: Arc<dyn TaskManager>,
}

impl TaskListTool {
    pub fn new(manager: Arc<dyn TaskManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl Tool for TaskListTool {
    fn name(&self) -> &str {
        "task_list"
    }

    fn description(&self) -> &str {
        "List all active (non-deleted) tasks. Returns a JSON array of task objects."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: false, is_list: true }
    }

    async fn execute(&self, _input: Value) -> AppResult<String> {
        let tasks = self.manager.list().await?;

        serde_json::to_string_pretty(&tasks)
            .map_err(|e| AppError::Tool(format!("failed to serialize tasks: {e}")))
    }
}
