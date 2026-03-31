mod infrastructure;

use std::sync::Arc;

use claude_rust_engine::QueryEngine;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{BashTool, ReadTool, ToolRegistry};
use claude_rust_types::AllowAll;

use infrastructure::http::handlers::AppState;
use infrastructure::http::routes::build_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,claude_rust_provider=debug".into()),
        )
        .init();

    let credential = claude_rust_auth::resolve_credential().expect("no credentials found");
    let provider = Arc::new(AnthropicProvider::new(credential));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(AllowAll);
    let engine = Arc::new(QueryEngine::new(provider, registry, permission));

    let state = AppState { engine };
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    tracing::info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("server error");
}
