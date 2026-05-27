use async_trait::async_trait;
use stynx_code_errors::AppResult;
use stynx_code_types::Conversation;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub message_count: usize,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {

    async fn save(&self, session_id: Option<&str>, conversation: &Conversation) -> AppResult<String>;

    async fn load(&self, session_id: &str) -> AppResult<Option<Conversation>>;

    async fn load_latest(&self) -> AppResult<Option<Conversation>>;

    async fn list(&self) -> AppResult<Vec<SessionSummary>>;

    async fn new_session_id(&self) -> AppResult<String>;

    async fn set_current(&self, session_id: &str) -> AppResult<()>;

    async fn current(&self) -> AppResult<Option<String>>;

    async fn delete(&self, session_id: &str) -> AppResult<()>;

    async fn rename(&self, session_id: &str, title: &str) -> AppResult<()>;
}
