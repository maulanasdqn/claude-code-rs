use async_trait::async_trait;
use claude_rust_errors::AppResult;
use claude_rust_types::Conversation;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, conversation: &Conversation) -> AppResult<String>;
    async fn load_latest(&self) -> AppResult<Option<Conversation>>;
    async fn list(&self) -> AppResult<Vec<String>>;
}
