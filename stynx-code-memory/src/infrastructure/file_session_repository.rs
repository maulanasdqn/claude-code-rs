use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::Conversation;
use tokio::fs;

use crate::domain::{SessionRepository, SessionSummary};

pub struct FileSessionRepository {
    project_dir: PathBuf,
}

impl FileSessionRepository {
    pub fn new(cwd: &str) -> AppResult<Self> {
        let home = stynx_code_config::home_dir()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("cannot determine home directory")))?;
        let slug = project_slug(cwd);
        let project_dir = home.join(".stynx-code").join("projects").join(slug);
        Ok(Self { project_dir })
    }

    pub fn with_dir(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    fn sessions_dir(&self) -> PathBuf { self.project_dir.join("sessions") }
    fn current_pointer(&self) -> PathBuf { self.project_dir.join("current.txt") }
    fn titles_path(&self) -> PathBuf { self.project_dir.join("titles.json") }
    fn legacy_session_path(&self) -> PathBuf { self.project_dir.join("session.json") }
    fn session_path(&self, id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{id}.json"))
    }

    async fn ensure_layout(&self) -> AppResult<()> {
        fs::create_dir_all(self.sessions_dir())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("mkdir failed: {e}")))?;
        let legacy = self.legacy_session_path();
        let migrated_target = self.session_path("legacy");
        if fs::try_exists(&legacy).await.unwrap_or(false)
            && !fs::try_exists(&migrated_target).await.unwrap_or(false)
        {
            if let Err(e) = fs::rename(&legacy, &migrated_target).await {
                tracing::warn!(?legacy, ?migrated_target, error = %e, "legacy session migration failed");
            } else {
                let _ = fs::write(self.current_pointer(), b"legacy").await;
            }
        }
        Ok(())
    }

    async fn read_titles(&self) -> std::collections::HashMap<String, String> {
        match fs::read_to_string(self.titles_path()).await {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => std::collections::HashMap::new(),
        }
    }

    async fn write_titles(&self, titles: &std::collections::HashMap<String, String>) -> AppResult<()> {
        let json = serde_json::to_string_pretty(titles)?;
        fs::write(self.titles_path(), json)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write titles failed: {e}")))?;
        Ok(())
    }
}

fn project_slug(cwd: &str) -> String {
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

fn now_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{ms}")
}

fn derive_title(conv: &Conversation) -> String {
    for msg in &conv.messages {
        if matches!(msg.role, stynx_code_types::Role::User) {
            for block in &msg.content {
                if let stynx_code_types::ContentBlock::Text { text } = block {
                    let t = text.trim();
                    if !t.is_empty() {
                        let line = t.lines().next().unwrap_or("").trim();
                        let mut s: String = line.chars().take(60).collect();
                        if line.chars().count() > 60 {
                            s.push('…');
                        }
                        if !s.is_empty() { return s; }
                    }
                }
            }
        }
    }
    "Untitled session".to_string()
}

#[async_trait]
impl SessionRepository for FileSessionRepository {
    async fn save(&self, session_id: Option<&str>, conversation: &Conversation) -> AppResult<String> {
        self.ensure_layout().await?;
        let id = match session_id {
            Some(s) => s.to_string(),
            None => match self.current().await? {
                Some(s) => s,
                None => {
                    let id = now_id();
                    let _ = self.set_current(&id).await;
                    id
                }
            },
        };
        let json = serde_json::to_string_pretty(conversation)?;
        let path = self.session_path(&id);
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json.as_bytes())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write failed: {e}")))?;
        fs::rename(&tmp, &path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("rename failed: {e}")))?;
        Ok(id)
    }

    async fn load(&self, session_id: &str) -> AppResult<Option<Conversation>> {
        self.ensure_layout().await?;
        let path = self.session_path(session_id);
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("read failed: {e}")))?;
        let conversation: Conversation = serde_json::from_str(&data)?;
        Ok(Some(conversation))
    }

    async fn load_latest(&self) -> AppResult<Option<Conversation>> {
        self.ensure_layout().await?;
        if let Some(id) = self.current().await? {
            if let Some(c) = self.load(&id).await? {
                return Ok(Some(c));
            }
        }
        let list = self.list().await.unwrap_or_default();
        if let Some(first) = list.first() {
            return self.load(&first.id).await;
        }
        Ok(None)
    }

    async fn list(&self) -> AppResult<Vec<SessionSummary>> {
        self.ensure_layout().await?;
        let titles = self.read_titles().await;
        let mut out: Vec<SessionSummary> = Vec::new();
        let mut entries = match fs::read_dir(self.sessions_dir()).await {
            Ok(e) => e,
            Err(_) => return Ok(out),
        };
        loop {
            let entry = match entries.next_entry().await {
                Ok(Some(e)) => e,
                Ok(None) => break,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let meta = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };
            let updated_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let (title, message_count) = match fs::read_to_string(&path).await {
                Ok(s) => match serde_json::from_str::<Conversation>(&s) {
                    Ok(c) => {
                        let title = titles
                            .get(&id)
                            .cloned()
                            .unwrap_or_else(|| derive_title(&c));
                        (title, c.messages.len())
                    }
                    Err(_) => (titles.get(&id).cloned().unwrap_or_else(|| id.clone()), 0),
                },
                Err(_) => (titles.get(&id).cloned().unwrap_or_else(|| id.clone()), 0),
            };
            out.push(SessionSummary { id, title, updated_at, message_count });
        }
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out)
    }

    async fn new_session_id(&self) -> AppResult<String> {
        self.ensure_layout().await?;
        Ok(now_id())
    }

    async fn set_current(&self, session_id: &str) -> AppResult<()> {
        self.ensure_layout().await?;
        fs::write(self.current_pointer(), session_id.as_bytes())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write current failed: {e}")))?;
        Ok(())
    }

    async fn current(&self) -> AppResult<Option<String>> {
        match fs::read_to_string(self.current_pointer()).await {
            Ok(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() { Ok(None) } else { Ok(Some(trimmed.to_string())) }
            }
            Err(_) => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> AppResult<()> {
        let path = self.session_path(session_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path)
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("delete failed: {e}")))?;
        }
        if self.current().await?.as_deref() == Some(session_id) {
            let _ = fs::remove_file(self.current_pointer()).await;
        }
        let mut titles = self.read_titles().await;
        if titles.remove(session_id).is_some() {
            let _ = self.write_titles(&titles).await;
        }
        Ok(())
    }

    async fn rename(&self, session_id: &str, title: &str) -> AppResult<()> {
        let mut titles = self.read_titles().await;
        titles.insert(session_id.to_string(), title.to_string());
        self.write_titles(&titles).await
    }
}
