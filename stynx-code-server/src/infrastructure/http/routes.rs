use axum::Router;
use axum::routing::{get, post};

use super::handlers::{AppState, chat, health, stream_chat};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .route("/stream", post(stream_chat))
        .with_state(state)
}
