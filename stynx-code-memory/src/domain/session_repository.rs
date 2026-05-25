use async_trait::async_trait;
use stynx_code_errors::AppResult;
use stynx_code_types::Conversation;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, conversation: &Conversation) -> AppResult<String>;
    async fn load_latest(&self) -> AppResult<Option<Conversation>>;
    async fn list(&self) -> AppResult<Vec<String>>;
}
