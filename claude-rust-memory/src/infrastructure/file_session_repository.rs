use std::path::PathBuf;

use async_trait::async_trait;
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::Conversation;
use tokio::fs;

use crate::domain::SessionRepository;

pub struct FileSessionRepository {
    project_dir: PathBuf,
}

impl FileSessionRepository {
    pub fn new(cwd: &str) -> AppResult<Self> {
        let home = claude_rust_config::home_dir()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("cannot determine home directory")))?;
        let slug = project_slug(cwd);
        let project_dir = home
            .join(".claude-code-rs")
            .join("projects")
            .join(slug);
        Ok(Self { project_dir })
    }

    pub fn with_dir(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    fn session_path(&self) -> PathBuf {
        self.project_dir.join("session.json")
    }

    fn tmp_path(&self) -> PathBuf {
        self.project_dir.join("session.json.tmp")
    }
}

fn project_slug(cwd: &str) -> String {
    // Strip leading `/` (Unix) or drive prefix like `C:\` (Windows)
    let stripped = cwd
        .trim_start_matches('/')
        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .trim_start_matches(':')
        .trim_start_matches(['/', '\\']);
    stripped
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .take(80)
        .collect()
}

#[async_trait]
impl SessionRepository for FileSessionRepository {
    async fn save(&self, conversation: &Conversation) -> AppResult<String> {
        fs::create_dir_all(&self.project_dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("mkdir failed: {e}")))?;

        let json = serde_json::to_string_pretty(conversation)?;
        let tmp = self.tmp_path();
        let target = self.session_path();

        fs::write(&tmp, json.as_bytes())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write failed: {e}")))?;
        fs::rename(&tmp, &target)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("rename failed: {e}")))?;

        Ok("current".to_string())
    }

    async fn load_latest(&self) -> AppResult<Option<Conversation>> {
        let path = self.session_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("read failed: {e}")))?;
        let conversation: Conversation = serde_json::from_str(&data)?;
        Ok(Some(conversation))
    }

    async fn list(&self) -> AppResult<Vec<String>> {
        if fs::try_exists(&self.session_path()).await.unwrap_or(false) {
            Ok(vec!["current".to_string()])
        } else {
            Ok(Vec::new())
        }
    }
}
