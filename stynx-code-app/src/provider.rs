use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_auth::Credential;
use stynx_code_config::{InternConfig, Settings};
use stynx_code_provider::{AnthropicProvider, OpenAiProvider};
use stynx_code_types::Provider;

pub struct ResolvedProvider {
    pub provider: Arc<dyn Provider>,
    pub anthropic: Option<Arc<AnthropicProvider>>,
    pub label: String,
}

pub struct InternInfo {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub description: String,
    pub key_env: String,
    pub available: bool,
}

pub fn list_interns_current() -> Vec<InternInfo> {
    list_interns(&stynx_code_config::load_config())
}

pub fn list_interns(config: &Settings) -> Vec<InternInfo> {
    let mut out: Vec<InternInfo> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for cfg in &config.interns {
        if seen.insert(cfg.name.to_lowercase()) {
            out.push(intern_info(
                cfg,
                cfg.description
                    .clone()
                    .unwrap_or_else(|| default_intern_description(&cfg.provider)),
            ));
        }
    }

    for cfg in legacy_env_interns() {
        if seen.insert(cfg.name.to_lowercase()) {
            let description = default_intern_description(&cfg.provider);
            out.push(intern_info(&cfg, description));
        }
    }

    out
}

fn intern_info(cfg: &InternConfig, description: String) -> InternInfo {
    InternInfo {
        name: cfg.name.clone(),
        provider: cfg.provider.clone(),
        model: cfg.model.clone(),
        description,
        key_env: intern_key_env(cfg),
        available: intern_api_key(cfg).is_some(),
    }
}

fn intern_key_env(cfg: &InternConfig) -> String {
    let (_, default_key_env, _) = provider_defaults(&cfg.provider);
    cfg.api_key_env
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_key_env.into())
}

fn default_intern_description(provider: &str) -> String {
    match provider.trim().to_lowercase().as_str() {
        "deepseek" => "Reasoning-heavy logic, tricky debugging, analysis",
        "qwen" => "General-purpose coding, refactors, drafting",
        "mimo" => "Mechanical / grunt work",
        "openrouter" => "Routed models via OpenRouter",
        "openai" => "OpenAI GPT models",
        _ => "OpenAI-compatible intern",
    }
    .to_string()
}

enum Candidate {
    Claude,
    Intern(InternConfig),
}

fn candidates(config: &Settings, claude_available: bool) -> Vec<(String, Candidate)> {
    let mut out: Vec<(String, Candidate)> = Vec::new();
    if claude_available {
        out.push(("claude".to_string(), Candidate::Claude));
    }
    for cfg in crate::interns::resolve_main_intern_candidates(config) {
        out.push((cfg.name.clone(), Candidate::Intern(cfg)));
    }
    out
}

pub fn list_main_providers() -> Vec<String> {
    let config = stynx_code_config::load_config();
    let claude_available = stynx_code_auth::resolve_credential().is_ok();
    candidates(&config, claude_available)
        .into_iter()
        .map(|(label, _)| label)
        .collect()
}

fn pick_index(
    override_label: Option<&str>,
    config: &Settings,
    list: &[(String, Candidate)],
) -> usize {
    let configured = override_label
        .map(str::to_string)
        .or_else(|| std::env::var("STYNX_MAIN_PROVIDER").ok())
        .or_else(|| config.main_provider.clone());
    if let Some(name) = configured.as_deref().map(str::trim).filter(|s| !s.is_empty())
        && let Some(i) = list.iter().position(|(label, _)| label.eq_ignore_ascii_case(name))
    {
        return i;
    }
    0
}

pub fn resolve_provider(
    config: &Settings,
    credential: Option<Credential>,
    mode_flag: Arc<AtomicU8>,
    override_label: Option<&str>,
) -> Result<ResolvedProvider, String> {
    let list = candidates(config, credential.is_some());
    if list.is_empty() {
        return Err(
            "No credentials found. Configure Claude (ANTHROPIC_API_KEY / OAuth login) or an \
             intern provider (e.g. DEEPSEEK_API_KEY, QWEN_API_KEY, OPENROUTER_API_KEY)."
                .to_string(),
        );
    }
    let idx = pick_index(override_label, config, &list);
    let (label, candidate) = &list[idx];

    match candidate {
        Candidate::Claude => {
            let cred = credential.ok_or("claude selected but credential missing")?;
            let provider = Arc::new(AnthropicProvider::new(cred, mode_flag));
            Ok(ResolvedProvider {
                provider: provider.clone(),
                anthropic: Some(provider),
                label: label.clone(),
            })
        }
        Candidate::Intern(cfg) => {
            let provider = build_openai_provider(cfg)?;
            Ok(ResolvedProvider { provider, anthropic: None, label: label.clone() })
        }
    }
}

fn provider_defaults(provider: &str) -> (&'static str, &'static str, &'static str) {
    match provider.trim().to_lowercase().as_str() {
        "deepseek" => ("https://api.deepseek.com/v1", "DEEPSEEK_API_KEY", "deepseek"),
        "openrouter" => ("https://openrouter.ai/api/v1", "OPENROUTER_API_KEY", "openrouter"),
        "openai" => ("https://api.openai.com/v1", "OPENAI_API_KEY", "openai"),
        "qwen" => (
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            "QWEN_API_KEY",
            "qwen",
        ),
        "mimo" | "custom" => ("https://api.xiaomimimo.com/v1", "MIMO_API_KEY", "custom"),
        _ => ("", "", "custom"),
    }
}

fn intern_api_key(cfg: &InternConfig) -> Option<String> {
    let (_, default_key_env, _) = provider_defaults(&cfg.provider);
    let key_env = cfg
        .api_key_env
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_key_env.into());
    std::env::var(&key_env).ok().filter(|s| !s.trim().is_empty())
}

fn build_openai_provider(cfg: &InternConfig) -> Result<Arc<dyn Provider>, String> {
    let (default_base, default_key_env, provider_label) = provider_defaults(&cfg.provider);
    let base_url = cfg
        .base_url
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_base.into());
    let key_env = cfg
        .api_key_env
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_key_env.into());
    let api_key = std::env::var(&key_env)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| format!("provider '{}' selected but {} is not set", cfg.name, key_env))?;
    Ok(Arc::new(OpenAiProvider::new(
        provider_label,
        base_url,
        api_key,
        &cfg.model,
    )))
}

fn legacy_env_interns() -> Vec<InternConfig> {
    [
        ("deepseek", "deepseek-v4-pro"),
        ("qwen", "qwen-plus"),
        ("mimo", "mimo-v2.5-pro"),
        ("openrouter", "openrouter/auto"),
        ("openai", "gpt-4o-mini"),
    ]
    .into_iter()
    .map(|(provider, model)| InternConfig {
        name: provider.to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        description: None,
        base_url: None,
        api_key_env: None,
    })
    .collect()
}
