use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::task::{CoordinatorTask, TaskState, TaskType};

pub struct CoordinatorTaskManager {
    tasks: Arc<RwLock<HashMap<String, CoordinatorTask>>>,
}

impl CoordinatorTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_task(&self, task_type: TaskType, desc: impl Into<String>) -> String {
        let id = next_id();
        let task = CoordinatorTask {
            id: id.clone(),
            task_type,
            state: TaskState::Pending,
            description: desc.into(),
            output: String::new(),
        };
        self.tasks.write().await.insert(id.clone(), task);
        id
    }

    pub async fn get_task(&self, id: &str) -> Option<CoordinatorTask> {
        self.tasks.read().await.get(id).cloned()
    }

    pub async fn list_tasks(&self) -> Vec<CoordinatorTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    pub async fn stop_task(&self, id: &str) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.state = TaskState::Stopped;
        }
    }

    pub async fn get_output(&self, id: &str) -> Option<String> {
        self.tasks.read().await.get(id).map(|t| t.output.clone())
    }
}

impl Default for CoordinatorTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("task-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}
