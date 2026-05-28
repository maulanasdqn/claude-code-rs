mod infrastructure;

use std::sync::Arc;

use stynx_code_config::{HooksConfig, PermissionSettings};
use stynx_code_engine::QueryEngine;
use stynx_code_permission::ConfigAwarePermissionChecker;
use stynx_code_provider::AnthropicProvider;
use stynx_code_tools::{BashTool, ReadTool, ToolRegistry};

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

    let server_token = std::env::var("STYNX_SERVER_TOKEN")
        .or_else(|_| std::env::var("CLAUDE_SERVER_TOKEN"))
        .ok()
        .filter(|s| !s.trim().is_empty());

    if server_token.is_none() {
        eprintln!(
            "FATAL: STYNX_SERVER_TOKEN (or legacy CLAUDE_SERVER_TOKEN) must be set. \
             Refusing to start an unauthenticated server."
        );
        std::process::exit(1);
    }

    let credential = stynx_code_auth::resolve_credential().expect("no credentials found");
    let mode = Arc::new(std::sync::atomic::AtomicU8::new(0));
    let provider = Arc::new(AnthropicProvider::new(credential.clone(), mode.clone()));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new()));
    registry.register(Arc::new(ReadTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(ConfigAwarePermissionChecker::new(
        PermissionSettings::default(),
        mode.clone(),
    ));
    let engine = Arc::new(QueryEngine::new(provider.clone(), registry, permission, mode, HooksConfig::default()));

    let started_at = std::time::Instant::now();
    let model_name = provider.model_name();
    let auth_type = if credential.is_oauth() { "oauth" } else { "api_key" };

    let state = AppState {
        engine,
        server_token,
        started_at,
        model_name,
        auth_type: auth_type.to_string(),
    };
    let app = build_router(state);

    let bind = std::env::var("STYNX_SERVER_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {bind}: {e}"));

    tracing::info!(bind = %bind, "stynx server listening");
    axum::serve(listener, app).await.expect("server error");
}
