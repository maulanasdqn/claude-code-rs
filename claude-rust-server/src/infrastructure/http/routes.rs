use axum::Router;
use axum::routing::{get, post};

use super::handlers::{AppState, chat, health};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .with_state(state)
}
