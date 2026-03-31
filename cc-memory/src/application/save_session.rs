use std::sync::Arc;

use cc_errors::AppResult;
use cc_types::Conversation;

use crate::domain::SessionRepository;

pub async fn save_session(
    repo: &Arc<dyn SessionRepository>,
    conversation: &Conversation,
) -> AppResult<String> {
    repo.save(conversation).await
}
