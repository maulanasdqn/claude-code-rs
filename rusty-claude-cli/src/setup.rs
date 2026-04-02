use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_config::load_config;
use claude_rust_engine::QueryEngine;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{
    AskUserTool, BashTool, ExitPlanModeTool, FileEditTool, FileWriteTool, GlobTool, GrepTool,
    ReadTool, TodoReadTool, TodoWriteTool, ToolRegistry, WebFetchTool, WebSearchTool,
};
use claude_rust_types::PermissionMode;

use crate::cli::Cli;

pub struct AppContext {
    pub engine: Arc<QueryEngine>,
    pub provider: Arc<AnthropicProvider>,
    pub cwd: String,
}

pub async fn build_context(cli: &Cli) -> Result<AppContext, String> {
    let config = load_config();
    let credential = claude_rust_auth::resolve_credential()
        .map_err(|e| format!("auth failed: {e}"))?;

    let mode_flag = Arc::new(AtomicU8::new(PermissionMode::Normal as u8));
    let provider = Arc::new(AnthropicProvider::new(credential, mode_flag.clone()));

    if let Some(ref model) = cli.model {
        provider.set_model(model);
    } else if let Some(ref model) = config.model {
        provider.set_model(model);
    }
    if let Ok(model) = std::env::var("MODEL") {
        provider.set_model(&model);
    }
    if let Some(mt) = config.max_tokens {
        provider.set_max_tokens(mt);
    }

    let pause_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(FileWriteTool));
    registry.register(Arc::new(FileEditTool));
    registry.register(Arc::new(GlobTool));
    registry.register(Arc::new(GrepTool));
    registry.register(Arc::new(AskUserTool::new(pause_flag.clone())));
    registry.register(Arc::new(WebFetchTool));
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(ExitPlanModeTool::new(pause_flag.clone())));
    registry.register(Arc::new(TodoWriteTool));
    registry.register(Arc::new(TodoReadTool));

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".into());

    for tool in claude_rust_tools::load_mcp_tools(&cwd).await {
        registry.register(tool);
    }

    let permission = Arc::new(ConfigAwarePermissionChecker::new_with_pause(
        config.permissions.clone(),
        mode_flag.clone(),
        pause_flag,
    ));

    let registry = Arc::new(registry);
    let mut engine = QueryEngine::new(
        provider.clone(),
        registry,
        permission,
        mode_flag.clone(),
        config.hooks.clone(),
    );
    engine = engine.with_max_turns(cli.max_turns);

    Ok(AppContext {
        engine: Arc::new(engine),
        provider,
        cwd,
    })
}
