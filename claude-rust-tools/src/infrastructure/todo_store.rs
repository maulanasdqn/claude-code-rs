use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: String,
    pub priority: String,
}

static TODOS: OnceLock<Mutex<Vec<TodoItem>>> = OnceLock::new();

fn store() -> &'static Mutex<Vec<TodoItem>> {
    TODOS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn read_todos() -> Vec<TodoItem> {
    store().lock().unwrap().clone()
}

pub fn write_todos(todos: Vec<TodoItem>) {
    *store().lock().unwrap() = todos;
}
