use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use cc_engine::QueryEngine;
use cc_errors::AppResult;
use cc_types::{ContentBlock, Conversation, Message};

use super::dto::{ChatRequest, ChatResponse};

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<QueryEngine>,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> AppResult<Json<ChatResponse>> {
    let mut conversation = Conversation::default();
    conversation.system = req.system;

    for msg in &req.messages {
        let message = match msg.role.as_str() {
            "user" => Message::user(&msg.content),
            _ => {
                return Err(cc_errors::AppError::BadRequest(
                    "only 'user' role is supported in request".into(),
                ));
            }
        };
        conversation.push(message);
    }

    let result = state.engine.run(conversation, |_| {}).await?;

    let response_text = result
        .messages
        .iter()
        .rev()
        .find(|m| matches!(m.role, cc_types::Role::Assistant))
        .map(|m| {
            m.content
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let messages = serde_json::to_value(&result.messages).unwrap_or_default();

    Ok(Json(ChatResponse {
        response: response_text,
        messages,
    }))
}
