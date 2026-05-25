use std::sync::Arc;

use stynx_code_errors::AppResult;
use stynx_code_types::Conversation;

use crate::domain::SessionRepository;

pub async fn load_session(
    repo: &Arc<dyn SessionRepository>,
) -> AppResult<Option<Conversation>> {
    repo.load_latest().await
}
