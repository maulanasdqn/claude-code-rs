mod infrastructure;

use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_config::load_config;
use claude_rust_engine::QueryEngine;
use claude_rust_memory::FileSessionRepository;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{
    AskUserTool, BashTool, EnterPlanModeTool, ExitPlanModeTool, FileEditTool, FileWriteTool,
    GlobTool, GrepTool, ReadTool, ToolRegistry, WebFetchTool, WebSearchTool,
};
use claude_rust_types::{Conversation, PermissionMode};

use infrastructure::app_loop::run_loop;
use infrastructure::event_renderer::render_error_box;
use infrastructure::terminal::{DIM, RESET, make_system_prompt, print_banner, prompt_resume};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(io::stderr)
        .init();

    let config = load_config();

    let credential = match claude_rust_auth::resolve_credential() {
        Ok(cred) => cred,
        Err(e) => { render_error_box(&e.to_string()); std::process::exit(1); }
    };

    let mode_flag = Arc::new(AtomicU8::new(PermissionMode::Normal as u8));
    let provider = Arc::new(AnthropicProvider::new(credential, mode_flag.clone()));

    if let Some(ref model) = config.model {
        provider.set_model(model);
    }

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(FileWriteTool));
    registry.register(Arc::new(FileEditTool));
    registry.register(Arc::new(GlobTool));
    registry.register(Arc::new(GrepTool));
    registry.register(Arc::new(AskUserTool));
    registry.register(Arc::new(WebFetchTool));
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(EnterPlanModeTool));
    registry.register(Arc::new(ExitPlanModeTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(ConfigAwarePermissionChecker::new(
        config.permissions.clone(),
        mode_flag.clone(),
    ));
    let engine = Arc::new(QueryEngine::new(provider.clone(), registry, permission, mode_flag.clone()));

    let session_repo: Arc<dyn claude_rust_memory::SessionRepository> =
        match FileSessionRepository::new() {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                tracing::warn!("failed to init session repository: {e}");
                Arc::new(FileSessionRepository::with_dir(
                    std::path::PathBuf::from(".claude-code-rs/sessions"),
                ))
            }
        };

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".into());

    print_banner(&cwd);

    let system_prompt = make_system_prompt(&cwd);

    let conversation = match claude_rust_memory::load_session(&session_repo).await {
        Ok(Some(prev)) if !prev.messages.is_empty() => {
            if prompt_resume() {
                let mut c = prev;
                if c.system.is_none() {
                    c.system = Some(system_prompt.clone());
                }
                println!("  {DIM}↻ Session resumed ({} messages){RESET}\n", c.messages.len());
                c
            } else {
                let mut c = Conversation::default();
                c.system = Some(system_prompt.clone());
                c
            }
        }
        _ => {
            let mut c = Conversation::default();
            c.system = Some(system_prompt);
            c
        }
    };

    run_loop(engine, session_repo, provider, config, mode_flag, cwd, conversation).await;
}
