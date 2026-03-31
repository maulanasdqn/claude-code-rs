use std::sync::Arc;

use cc_errors::AppResult;
use cc_types::Conversation;

use crate::domain::SessionRepository;

pub async fn load_session(
    repo: &Arc<dyn SessionRepository>,
) -> AppResult<Option<Conversation>> {
    repo.load_latest().await
}
