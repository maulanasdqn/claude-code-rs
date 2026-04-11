use async_trait::async_trait;
use claude_rust_errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub created_at: u64,
    pub tags: Vec<String>,
}

#[async_trait]
pub trait SessionMemoryService: Send + Sync {
    async fn extract_memories(&self, conversation: &str) -> Vec<Memory>;
    async fn save_memory(&self, memory: &Memory) -> AppResult<()>;
    async fn load_memories(&self) -> AppResult<Vec<Memory>>;
}

pub struct FileSessionMemory {
    dir: PathBuf,
}

impl FileSessionMemory {
    pub fn new() -> Self {
        let dir = home_claude_dir().join("memories");
        Self { dir }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }
}

fn home_claude_dir() -> PathBuf {
    claude_rust_config::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[async_trait]
impl SessionMemoryService for FileSessionMemory {
    async fn extract_memories(&self, conversation: &str) -> Vec<Memory> {
        // Simple extraction: each non-empty line that starts with "REMEMBER:" becomes a memory.
        conversation
            .lines()
            .filter(|line| line.starts_with("REMEMBER:"))
            .map(|line| {
                let content = line.trim_start_matches("REMEMBER:").trim().to_string();
                Memory {
                    id: format!("mem_{}", now_secs()),
                    content,
                    created_at: now_secs(),
                    tags: Vec::new(),
                }
            })
            .collect()
    }

    async fn save_memory(&self, memory: &Memory) -> AppResult<()> {
        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(|e| -> AppError {
                anyhow::anyhow!("failed to create memories dir: {e}").into()
            })?;

        let path = self.dir.join(format!("{}.json", memory.id));
        let data = serde_json::to_string_pretty(memory)?;

        tokio::fs::write(&path, data)
            .await
            .map_err(|e| -> AppError {
                anyhow::anyhow!("failed to write memory file: {e}").into()
            })?;

        tracing::info!(id = %memory.id, "saved memory");
        Ok(())
    }

    async fn load_memories(&self) -> AppResult<Vec<Memory>> {
        let mut memories = Vec::new();

        let dir_exists = tokio::fs::metadata(&self.dir).await.is_ok();
        if !dir_exists {
            return Ok(memories);
        }

        let mut entries = tokio::fs::read_dir(&self.dir)
            .await
            .map_err(|e| -> AppError {
                anyhow::anyhow!("failed to read memories dir: {e}").into()
            })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| -> AppError {
                anyhow::anyhow!("failed to read dir entry: {e}").into()
            })?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let data = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| -> AppError {
                        anyhow::anyhow!("failed to read memory file: {e}").into()
                    })?;
                if let Ok(memory) = serde_json::from_str::<Memory>(&data) {
                    memories.push(memory);
                } else {
                    tracing::warn!(path = %path.display(), "skipping malformed memory file");
                }
            }
        }

        memories.sort_by_key(|m| m.created_at);
        Ok(memories)
    }
}
