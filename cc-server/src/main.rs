use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Json, Router};
use cc_engine::QueryEngine;
use cc_errors::AppResult;
use cc_provider::AnthropicProvider;
use cc_tools::{BashTool, ReadTool, ToolRegistry};
use cc_types::{AllowAll, ContentBlock, Conversation, Message};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
struct AppState {
    engine: Arc<QueryEngine>,
}

#[derive(Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    #[serde(default)]
    system: Option<String>,
}

#[derive(Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    messages: Value,
}

async fn health() -> &'static str {
    "ok"
}

async fn chat(
    axum::extract::State(state): axum::extract::State<AppState>,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cc_provider=debug".into()),
        )
        .init();

    let credential = cc_auth::resolve_credential().expect("no credentials found");
    let provider = Arc::new(AnthropicProvider::new(credential));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(AllowAll);

    let engine = Arc::new(QueryEngine::new(provider, registry, permission));

    let state = AppState { engine };

    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    tracing::info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
