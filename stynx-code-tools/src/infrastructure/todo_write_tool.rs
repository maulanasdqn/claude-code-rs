use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::todo_store::{TodoItem, write_todos};

pub struct TodoWriteTool;

#[async_trait::async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> &str {
        "todo_write"
    }

    fn description(&self) -> &str {
        "Create and manage a structured task list. Use to track tasks, their status, and priorities. Replaces the entire todo list with the provided items."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "The complete list of todo items to store",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique identifier for the todo item"
                            },
                            "content": {
                                "type": "string",
                                "description": "Description of the task"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "Current status of the task"
                            },
                            "priority": {
                                "type": "string",
                                "enum": ["high", "medium", "low"],
                                "description": "Priority level of the task"
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "Present continuous form for spinner display (e.g., 'Implementing feature')"
                            }
                        },
                        "required": ["id", "content", "status", "priority"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let arr = input
            .get("todos")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AppError::Tool("missing 'todos' array".into()))?;

        let todos: Vec<TodoItem> = arr
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();

        let count = todos.len();
        write_todos(todos);
        Ok(format!("Updated todo list ({count} items)"))
    }
}
