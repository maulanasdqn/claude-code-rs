use std::sync::Arc;

use claude_rust_errors::AppResult;
use claude_rust_types::Conversation;

use crate::domain::SessionRepository;

pub async fn load_session(
    repo: &Arc<dyn SessionRepository>,
) -> AppResult<Option<Conversation>> {
    repo.load_latest().await
}
