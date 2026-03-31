use std::path::PathBuf;
use std::time::SystemTime;

use async_trait::async_trait;
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::Conversation;
use tokio::fs;

use crate::domain::SessionRepository;

pub struct FileSessionRepository {
    sessions_dir: PathBuf,
}

impl FileSessionRepository {
    pub fn new() -> AppResult<Self> {
        let home = std::env::var("HOME")
            .map_err(|_| AppError::Internal(anyhow::anyhow!("HOME not set")))?;
        let sessions_dir = PathBuf::from(home).join(".claude-code-rs").join("sessions");
        Ok(Self { sessions_dir })
    }

    pub fn with_dir(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{id}.json"))
    }

    fn tmp_path(&self, id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{id}.json.tmp"))
    }

    fn generate_id() -> AppResult<String> {
        let millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("SystemTime error: {e}")))?
            .as_millis();
        Ok(format!("{millis}"))
    }
}

#[async_trait]
impl SessionRepository for FileSessionRepository {
    async fn save(&self, conversation: &Conversation) -> AppResult<String> {
        fs::create_dir_all(&self.sessions_dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to create sessions dir: {e}")))?;

        let id = Self::generate_id()?;
        let json = serde_json::to_string_pretty(conversation)?;

        let tmp = self.tmp_path(&id);
        let target = self.session_path(&id);

        fs::write(&tmp, json.as_bytes())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to write tmp file: {e}")))?;

        fs::rename(&tmp, &target)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to rename session file: {e}")))?;

        Ok(id)
    }

    async fn load_latest(&self) -> AppResult<Option<Conversation>> {
        let ids = self.list().await?;
        let latest = match ids.last() {
            Some(id) => id,
            None => return Ok(None),
        };

        let path = self.session_path(latest);
        let data = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to read session: {e}")))?;

        let conversation: Conversation = serde_json::from_str(&data)?;
        Ok(Some(conversation))
    }

    async fn list(&self) -> AppResult<Vec<String>> {
        let exists = fs::try_exists(&self.sessions_dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to check sessions dir: {e}")))?;

        if !exists {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&self.sessions_dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to read sessions dir: {e}")))?;

        let mut ids = Vec::new();
        loop {
            let entry = entries
                .next_entry()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to read dir entry: {e}")))?;

            match entry {
                Some(entry) => {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if let Some(id) = name.strip_suffix(".json") {
                        if !id.ends_with(".tmp") {
                            ids.push(id.to_string());
                        }
                    }
                }
                None => break,
            }
        }

        ids.sort();
        Ok(ids)
    }
}
