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
    /// Save a conversation under the given session id (or the current id when
    /// `None`). Returns the id used. The session is marked "current" on the
    /// next call to `set_current`, not implicitly.
    async fn save(&self, session_id: Option<&str>, conversation: &Conversation) -> AppResult<String>;

    /// Load a specific session by id.
    async fn load(&self, session_id: &str) -> AppResult<Option<Conversation>>;

    /// Load the most recently-active session (whatever `current` points at).
    async fn load_latest(&self) -> AppResult<Option<Conversation>>;

    /// Return all sessions for this project, newest first.
    async fn list(&self) -> AppResult<Vec<SessionSummary>>;

    /// Generate a brand-new session id. Implementations should make this
    /// monotonic so the new session sorts to the top of the list.
    async fn new_session_id(&self) -> AppResult<String>;

    /// Update the "current" pointer.
    async fn set_current(&self, session_id: &str) -> AppResult<()>;

    /// Get the current session id, if any.
    async fn current(&self) -> AppResult<Option<String>>;

    /// Delete a session by id.
    async fn delete(&self, session_id: &str) -> AppResult<()>;

    /// Rename a session.
    async fn rename(&self, session_id: &str, title: &str) -> AppResult<()>;
}
