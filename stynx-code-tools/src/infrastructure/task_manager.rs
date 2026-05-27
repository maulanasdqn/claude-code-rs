use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use stynx_code_errors::{AppError, AppResult};
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Deleted,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}

#[async_trait::async_trait]
pub trait TaskManager: Send + Sync {
    async fn create(&self, subject: &str, description: &str) -> AppResult<TaskInfo>;
    async fn get(&self, task_id: &str) -> AppResult<TaskInfo>;
    async fn list(&self) -> AppResult<Vec<TaskInfo>>;
    async fn update(&self, task_id: &str, updates: Value) -> AppResult<TaskInfo>;
    async fn stop(&self, task_id: &str) -> AppResult<()>;
    async fn output(&self, task_id: &str) -> AppResult<String>;
}

pub struct InMemoryTaskManager {
    tasks: tokio::sync::RwLock<Vec<TaskInfo>>,
    next_id: AtomicU64,
}

impl InMemoryTaskManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            tasks: tokio::sync::RwLock::new(Vec::new()),
            next_id: AtomicU64::new(1),
        })
    }
}

#[async_trait::async_trait]
impl TaskManager for InMemoryTaskManager {
    async fn create(&self, subject: &str, description: &str) -> AppResult<TaskInfo> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let task = TaskInfo {
            id: id.to_string(),
            subject: subject.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            owner: None,
            blocked_by: Vec::new(),
            metadata: Value::Null,
        };
        self.tasks.write().await.push(task.clone());
        Ok(task)
    }

    async fn get(&self, task_id: &str) -> AppResult<TaskInfo> {
        self.tasks
            .read()
            .await
            .iter()
            .find(|t| t.id == task_id)
            .cloned()
            .ok_or_else(|| AppError::Tool(format!("task not found: {task_id}")))
    }

    async fn list(&self) -> AppResult<Vec<TaskInfo>> {
        let tasks = self.tasks.read().await;
        Ok(tasks
            .iter()
            .filter(|t| t.status != TaskStatus::Deleted)
            .cloned()
            .collect())
    }

    async fn update(&self, task_id: &str, updates: Value) -> AppResult<TaskInfo> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::Tool(format!("task not found: {task_id}")))?;

        if let Some(s) = updates.get("status").and_then(|v| v.as_str()) {
            task.status = match s {
                "pending" => TaskStatus::Pending,
                "in_progress" => TaskStatus::InProgress,
                "completed" => TaskStatus::Completed,
                "deleted" => TaskStatus::Deleted,
                other => return Err(AppError::Tool(format!("invalid status: {other}"))),
            };
        }
        if let Some(s) = updates.get("subject").and_then(|v| v.as_str()) {
            task.subject = s.to_string();
        }
        if let Some(s) = updates.get("description").and_then(|v| v.as_str()) {
            task.description = s.to_string();
        }
        if let Some(s) = updates.get("owner").and_then(|v| v.as_str()) {
            task.owner = Some(s.to_string());
        }
        if let Some(m) = updates.get("metadata") {
            task.metadata = m.clone();
        }

        Ok(task.clone())
    }

    async fn stop(&self, task_id: &str) -> AppResult<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::Tool(format!("task not found: {task_id}")))?;
        task.status = TaskStatus::Deleted;
        Ok(())
    }

    async fn output(&self, task_id: &str) -> AppResult<String> {

        self.tasks
            .read()
            .await
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| AppError::Tool(format!("task not found: {task_id}")))?;

        Ok(String::new())
    }
}
