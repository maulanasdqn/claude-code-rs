use std::sync::Arc;
use std::convert::Infallible;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{ContentBlock, Conversation, Message};
use futures::StreamExt;
use serde_json::json;

use super::dto::{ChatRequest, ChatResponse, StreamEventDto};

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<QueryEngine>,
    pub server_token: Option<String>,
    pub started_at: std::time::Instant,
    pub model_name: String,
    pub auth_type: String,
}

fn check_auth(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    if let Some(token) = &state.server_token {
        let provided = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));
        if provided != Some(token.as_str()) {
            return Err(AppError::Unauthorized);
        }
    }
    Ok(())
}

fn build_conversation(req: ChatRequest) -> Result<Conversation, AppError> {
    let mut conversation = Conversation::default();
    conversation.system = req.system;
    for msg in &req.messages {
        let message = match msg.role.as_str() {
            "user" => Message::user(&msg.content),
            _ => return Err(AppError::BadRequest("only 'user' role is supported".into())),
        };
        conversation.push(message);
    }
    Ok(conversation)
}

pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "uptime_seconds": state.started_at.elapsed().as_secs(),
        "model": state.model_name,
        "auth_type": state.auth_type,
    }))
}

pub async fn chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> AppResult<Json<ChatResponse>> {
    check_auth(&state, &headers)?;
    let conversation = build_conversation(req)?;
    let result = state.engine.run(conversation, |_| {}).await?;

    let response_text = result
        .messages
        .iter()
        .rev()
        .find(|m| matches!(m.role, claude_rust_types::Role::Assistant))
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
    Ok(Json(ChatResponse { response: response_text, messages }))
}

pub async fn stream_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> AppResult<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>> {
    check_auth(&state, &headers)?;
    let conversation = build_conversation(req)?;

    let (tx, rx) = futures::channel::mpsc::unbounded::<StreamEventDto>();
    let engine = state.engine.clone();
    tokio::spawn(async move {
        let _ = engine.run(conversation, move |event| {
            let dto = match event {
                EngineEvent::TextDelta(text) => Some(StreamEventDto::Content { text }),
                EngineEvent::ToolResult { name, output, is_error: false } => {
                    Some(StreamEventDto::ToolUse { name, output })
                }
                EngineEvent::TurnComplete => Some(StreamEventDto::Done),
                EngineEvent::Error(msg) => Some(StreamEventDto::Error { message: msg }),
                _ => None,
            };
            if let Some(d) = dto {
                let _ = tx.unbounded_send(d);
            }
        }).await;
    });

    let sse = rx.map(|dto| {
        let data = serde_json::to_string(&dto).unwrap_or_default();
        Ok::<Event, Infallible>(Event::default().data(data))
    });

    Ok(Sse::new(sse).keep_alive(KeepAlive::default()))
}
