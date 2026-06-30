use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8};

use stynx_code_config::{Settings, load_config};
use stynx_code_engine::QueryEngine;
use stynx_code_memory::{FileSessionRepository, SessionRepository};
use stynx_code_permission::ConfigAwarePermissionChecker;
use stynx_code_provider::AnthropicProvider;
use stynx_code_tools::{
    AskUserTool, BashTool, ExitPlanModeTool, FileEditTool, FileWriteTool, GlobTool, GrepTool,
    ReadTool, SharedQuestionBridge, TodoReadTool, TodoWriteTool, ToolRegistry, WebFetchTool,
    WebSearchTool,
};
use stynx_code_types::{PermissionMode, Provider};

use crate::agent_tool::{AgentTool, AllInternsTool, ExploreAgentTool};
use crate::intern_manager::InternManager;
use crate::intern_tools::{InternKillTool, InternStatusTool, InternWaitTool};
use crate::interns::build_intern_tools;
use crate::provider::resolve_provider;
use crate::system_prompt::{build_env_info, make_system_prompt};

pub struct AppOptions {
    pub cwd: String,
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
    pub system_override: Option<String>,
}

impl AppOptions {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self {
            cwd: cwd.into(),
            provider_override: None,
            model_override: None,
            system_override: None,
        }
    }
}

pub struct AppHandles {
    pub engine: Arc<QueryEngine>,
    pub provider: Arc<dyn Provider>,
    pub anthropic: Option<Arc<AnthropicProvider>>,
    pub mode_flag: Arc<AtomicU8>,
    pub pause_flag: Arc<AtomicBool>,
    pub permission: Arc<ConfigAwarePermissionChecker>,
    pub ask_user_bridge: SharedQuestionBridge,
    pub session_repo: Arc<dyn SessionRepository>,
    pub system_prompt: String,
    pub model_id: String,
    pub provider_label: String,
    pub workspace_bridge: crate::workspace_bridge::SharedWorkspaceBridge,
    pub config: Settings,
}

pub async fn build_app(options: AppOptions) -> Result<AppHandles, String> {
    crate::keys::apply_persisted_keys();
    let config = load_config();
    let credential = stynx_code_auth::resolve_credential().ok();

    let mode_flag = Arc::new(AtomicU8::new(PermissionMode::Normal as u8));
    let pause_flag = Arc::new(AtomicBool::new(false));

    let resolved = resolve_provider(
        &config,
        credential,
        mode_flag.clone(),
        options.provider_override.as_deref(),
    )?;
    let provider_label = resolved.label.clone();
    let provider = resolved.provider;

    apply_model_settings(&provider, &config, options.model_override.as_deref());

    let built = build_registry(&options.cwd, &pause_flag).await;
    let ask_user_bridge = built.ask_user_bridge.clone();
    let mut registry = built.registry;

    let permission = Arc::new(ConfigAwarePermissionChecker::new_with_pause(
        config.permissions.clone(),
        mode_flag.clone(),
        pause_flag.clone(),
    ));

    register_agents_and_interns(&mut registry, &provider, &permission, &mode_flag, &config);

    let workspace_bridge: crate::workspace_bridge::SharedWorkspaceBridge =
        std::sync::Arc::new(crate::workspace_bridge::OptionalWorkspaceBridge::new());
    registry.register(std::sync::Arc::new(crate::workspace_bridge::MessageWorkspaceTool::new(
        workspace_bridge.clone(),
    )));

    let tool_names = registry.tool_names();
    let registry = Arc::new(registry);

    let engine = Arc::new(QueryEngine::new(
        provider.clone(),
        registry,
        permission.clone(),
        mode_flag.clone(),
        config.hooks.clone(),
    ));

    let model_id = provider.model_name();
    let system_prompt = options.system_override.clone().unwrap_or_else(|| {
        let env = build_env_info(options.cwd.clone(), model_id.clone());
        make_system_prompt(&env, &tool_names, &[], config.commit_attribution)
    });

    let session_repo: Arc<dyn SessionRepository> = match FileSessionRepository::new(&options.cwd) {
        Ok(repo) => Arc::new(repo),
        Err(error) => {
            tracing::warn!("failed to init session repository: {error}");
            Arc::new(FileSessionRepository::with_dir(std::path::PathBuf::from(
                ".stynx-code/projects/fallback",
            )))
        }
    };

    Ok(AppHandles {
        engine,
        provider,
        anthropic: resolved.anthropic,
        mode_flag,
        pause_flag,
        permission,
        ask_user_bridge,
        session_repo,
        system_prompt,
        model_id,
        provider_label,
        workspace_bridge,
        config,
    })
}

fn apply_model_settings(provider: &Arc<dyn Provider>, config: &Settings, model_override: Option<&str>) {
    if let Some(model) = model_override {
        provider.set_model(model);
    } else if let Some(ref model) = config.model {
        provider.set_model(model);
    }
    if let Ok(model) = std::env::var("MODEL") {
        provider.set_model(&model);
    }
    if let Some(max_tokens) = config.max_tokens {
        provider.set_max_tokens(max_tokens);
    }
    if let Some(ref effort) = config.effort {
        let level = effort.to_lowercase();
        if ["low", "medium", "high", "max"].contains(&level.as_str()) {
            provider.set_effort(&level);
        }
    }
}

fn register_agents_and_interns(
    registry: &mut ToolRegistry,
    provider: &Arc<dyn Provider>,
    permission: &Arc<ConfigAwarePermissionChecker>,
    mode_flag: &Arc<AtomicU8>,
    config: &Settings,
) {
    let sub_registry = Arc::new(registry.clone_excluding(&["agent", "explore"]));
    let explore_registry = Arc::new(sub_registry.clone_excluding(&[
        "bash",
        "file_write",
        "file_edit",
        "ask_user_question",
        "web_fetch",
        "web_search",
        "todo_write",
        "todo_read",
    ]));

    registry.register(Arc::new(AgentTool::new(
        provider.clone(),
        sub_registry.clone(),
        permission.clone(),
        mode_flag.clone(),
        config.hooks.clone(),
    )));
    registry.register(Arc::new(ExploreAgentTool::new(
        provider.clone(),
        explore_registry,
        permission.clone(),
        mode_flag.clone(),
        config.hooks.clone(),
    )));

    let intern_manager = InternManager::new();
    let intern_tools = build_intern_tools(
        config,
        &sub_registry,
        permission,
        mode_flag,
        &config.hooks,
        &intern_manager,
    );
    for tool in &intern_tools {
        registry.register(tool.clone());
    }

    if !intern_tools.is_empty() {
        registry.register(Arc::new(InternStatusTool::new(intern_manager.clone())));
        registry.register(Arc::new(InternKillTool::new(intern_manager.clone())));
        registry.register(Arc::new(InternWaitTool::new(intern_manager.clone())));
    }
    if intern_tools.len() >= 2 {
        registry.register(Arc::new(AllInternsTool::new(intern_tools)));
    }
}

struct BuiltRegistry {
    registry: ToolRegistry,
    ask_user_bridge: SharedQuestionBridge,
}

async fn build_registry(cwd: &str, pause_flag: &Arc<AtomicBool>) -> BuiltRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool::new()));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(FileWriteTool));
    registry.register(Arc::new(FileEditTool));
    registry.register(Arc::new(GlobTool));
    registry.register(Arc::new(GrepTool));

    let ask_user_tool = Arc::new(AskUserTool::new(pause_flag.clone()));
    let ask_user_bridge = ask_user_tool.bridge_handle();
    registry.register(ask_user_tool);

    registry.register(Arc::new(WebFetchTool));
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(ExitPlanModeTool::new(pause_flag.clone())));
    registry.register(Arc::new(TodoWriteTool));
    registry.register(Arc::new(TodoReadTool));

    for tool in stynx_code_tools::load_mcp_tools(cwd).await {
        registry.register(tool);
    }

    BuiltRegistry { registry, ask_user_bridge }
}
