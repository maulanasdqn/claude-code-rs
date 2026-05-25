mod infrastructure;

use std::sync::Arc;

use stynx_code_config::HooksConfig;
use stynx_code_engine::QueryEngine;
use stynx_code_provider::AnthropicProvider;
use stynx_code_tools::{BashTool, ReadTool, ToolRegistry};
use stynx_code_types::AllowAll;

use infrastructure::http::handlers::AppState;
use infrastructure::http::routes::build_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let credential = stynx_code_auth::resolve_credential().expect("no credentials found");
    let mode = Arc::new(std::sync::atomic::AtomicU8::new(0));
    let provider = Arc::new(AnthropicProvider::new(credential.clone(), mode.clone()));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(AllowAll);
    let engine = Arc::new(QueryEngine::new(provider.clone(), registry, permission, mode, HooksConfig::default()));

    let started_at = std::time::Instant::now();
    let model_name = provider.model_name();
    let auth_type = if credential.is_oauth() { "oauth" } else { "api_key" };
    let server_token = std::env::var("CLAUDE_SERVER_TOKEN").ok();

    let state = AppState {
        engine,
        server_token,
        started_at,
        model_name,
        auth_type: auth_type.to_string(),
    };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    tracing::info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
