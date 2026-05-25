use std::sync::Arc;

use stynx_code_errors::AppResult;
use stynx_code_types::Conversation;

use crate::domain::SessionRepository;

pub async fn save_session(
    repo: &Arc<dyn SessionRepository>,
    conversation: &Conversation,
) -> AppResult<String> {
    repo.save(conversation).await
}
