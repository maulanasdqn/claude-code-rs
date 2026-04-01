mod infrastructure;

use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_config::load_config;
use claude_rust_engine::{QueryEngine, run_session_start_hooks};
use claude_rust_memory::FileSessionRepository;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{
    AskUserTool, BashTool, ExitPlanModeTool, FileEditTool, FileWriteTool,
    GlobTool, GrepTool, ReadTool, TodoReadTool, TodoWriteTool, ToolRegistry, WebFetchTool,
    WebSearchTool,
};
use claude_rust_types::{Conversation, PermissionMode};

use infrastructure::agent_tool::{AgentTool, ExploreAgentTool};
use infrastructure::app_loop::run_loop;
use infrastructure::event_renderer::render_error_box;
use infrastructure::skills::load_skills;
use infrastructure::terminal::{DIM, RESET, build_env_info, make_system_prompt, print_banner, prompt_resume};

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
        pause_flag.clone(),
    ));

    let sub_registry = Arc::new(registry.clone_excluding(&["agent", "explore"]));
    let explore_registry = Arc::new(sub_registry.clone_excluding(&[
        "bash", "file_write", "file_edit", "ask_user_question",
        "web_fetch", "web_search", "todo_write", "todo_read",
    ]));

    registry.register(Arc::new(AgentTool::new(
        provider.clone(), sub_registry, permission.clone(), mode_flag.clone(), config.hooks.clone(),
    )));
    registry.register(Arc::new(ExploreAgentTool::new(
        provider.clone(), explore_registry, permission.clone(), mode_flag.clone(), config.hooks.clone(),
    )));

    let tool_names = registry.tool_names();
    let registry = Arc::new(registry);
    let engine = {
        let mut e = QueryEngine::new(provider.clone(), registry, permission.clone(), mode_flag.clone(), config.hooks.clone());
        if let Some(mt) = config.max_turns {
            e = e.with_max_turns(mt);
        }
        Arc::new(e)
    };

    let session_repo: Arc<dyn claude_rust_memory::SessionRepository> =
        match FileSessionRepository::new(&cwd) {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                tracing::warn!("failed to init session repository: {e}");
                Arc::new(FileSessionRepository::with_dir(
                    std::path::PathBuf::from(".claude-code-rs/projects/fallback"),
                ))
            }
        };

    let session_start_out = run_session_start_hooks(&config.hooks).await;
    let model_id = provider.model_name();

    print_banner(&cwd, &model_id);

    if !session_start_out.is_empty() {
        println!("  {DIM}{session_start_out}{RESET}\n");
    }

    let env = build_env_info(cwd.clone(), model_id);
    let loaded_skills = load_skills(&cwd);
    let skill_pairs: Vec<(String, String)> = loaded_skills.iter()
        .map(|s| (s.name.clone(), s.description.clone()))
        .collect();
    let system_prompt = make_system_prompt(&env, &tool_names, &skill_pairs);
    tracing::debug!("system prompt: {} chars", system_prompt.len());

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
            c.system = Some(system_prompt.clone());
            c
        }
    };

    run_loop(engine, session_repo, provider, config, mode_flag, cwd, system_prompt, conversation, loaded_skills, pause_flag).await;
}
