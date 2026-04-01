use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::todo_store::read_todos;

pub struct TodoReadTool;

#[async_trait::async_trait]
impl Tool for TodoReadTool {
    fn name(&self) -> &str {
        "todo_read"
    }

    fn description(&self) -> &str {
        "Read the current task list to check pending, in-progress, and completed tasks."
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

    async fn execute(&self, _input: Value) -> AppResult<String> {
        let todos = read_todos();
        if todos.is_empty() {
            return Ok("No todos.".into());
        }
        let mut out = String::new();
        for t in &todos {
            let icon = match t.status.as_str() {
                "completed" => "✓",
                "in_progress" => "→",
                _ => "○",
            };
            let pri = match t.priority.as_str() {
                "high" => "[!]",
                "medium" => "[~]",
                _ => "[ ]",
            };
            out.push_str(&format!("{icon} {pri} [{}] {}\n", t.id, t.content));
        }
        Ok(out.trim_end().to_string())
    }
}
