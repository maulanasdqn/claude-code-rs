use std::sync::Arc;

use claude_rust_errors::AppResult;
use claude_rust_types::Conversation;

use crate::domain::SessionRepository;

pub async fn save_session(
    repo: &Arc<dyn SessionRepository>,
    conversation: &Conversation,
) -> AppResult<String> {
    repo.save(conversation).await
}
