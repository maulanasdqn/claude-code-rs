use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_permission::ConfigAwarePermissionChecker;
use stynx_code_provider::OpenAiProvider;
use stynx_code_tools::ToolRegistry;

use crate::agent_tool::InternTool;
use crate::intern_manager::InternManager;

const INTERN_TOOLS: &[&str] = &[
    "bash", "read", "file_write", "file_edit", "glob", "grep",
];

struct ResolvedIntern {
    name: String,
    provider_label: String,
    description: String,
    base_url: String,
    api_key: String,
    model: String,
}

pub fn build_intern_tools(
    config: &stynx_code_config::Settings,
    sub_registry: &Arc<ToolRegistry>,
    permission: &Arc<ConfigAwarePermissionChecker>,
    mode_flag: &Arc<AtomicU8>,
    hooks: &stynx_code_config::HooksConfig,
    manager: &Arc<InternManager>,
) -> Vec<Arc<InternTool>> {
    let resolved = resolve_interns(config);
    let intern_registry = Arc::new(sub_registry.keep_only(INTERN_TOOLS));

    let mut out: Vec<Arc<InternTool>> = Vec::new();
    let mut seen_tool_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for r in resolved {
        let tool_name = sanitize_tool_name(&format!("delegate_to_{}", r.name));
        if !seen_tool_names.insert(tool_name.clone()) {
            tracing::warn!(name = %r.name, "duplicate intern name, skipping");
            continue;
        }

        let intern_provider: Arc<dyn stynx_code_types::Provider> = Arc::new(
            OpenAiProvider::new(&r.provider_label, r.base_url.clone(), r.api_key.clone(), &r.model),
        );

        eprintln!(
            "  \x1b[2m· intern ready: {name} ({provider} / {model})\x1b[0m",
            name = r.name, provider = r.provider_label, model = r.model,
        );

        out.push(Arc::new(
            InternTool::new(
                intern_provider,
                intern_registry.clone(),
                permission.clone(),
                mode_flag.clone(),
                hooks.clone(),
                r.name.clone(),
                tool_name,
                r.description.clone(),
            ).with_manager(manager.clone()),
        ));
    }
    out
}

pub fn resolve_main_intern_candidates(config: &stynx_code_config::Settings) -> Vec<stynx_code_config::InternConfig> {
    let mut out: Vec<stynx_code_config::InternConfig> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for cfg in &config.interns {
        if resolve_one(cfg).is_some() && seen.insert(cfg.name.clone()) {
            out.push(cfg.clone());
        }
    }
    for r in resolve_interns(config) {
        if seen.insert(r.name.clone()) {
            out.push(stynx_code_config::InternConfig {
                name: r.name.clone(),
                provider: if r.provider_label == "custom" { "custom".into() } else { r.provider_label.clone() },
                model: r.model.clone(),
                description: None,
                base_url: Some(r.base_url.clone()),
                api_key_env: None,
            });
        }
    }
    out
}

fn resolve_interns(config: &stynx_code_config::Settings) -> Vec<ResolvedIntern> {
    let mut out: Vec<ResolvedIntern> = Vec::new();
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for cfg in &config.interns {
        if let Some(r) = resolve_one(cfg) {
            if names.insert(r.name.clone()) {
                out.push(r);
            }
        }
    }

    if !names.contains("deepseek") {
        if let Some(r) = legacy_deepseek_intern() {
            names.insert(r.name.clone());
            out.push(r);
        }
    }

    if !names.contains("qwen") {
        if let Some(r) = legacy_qwen_intern() {
            names.insert(r.name.clone());
            out.push(r);
        }
    }

    if !names.contains("mimo") {
        if let Some(r) = legacy_mimo_intern() {
            names.insert(r.name.clone());
            out.push(r);
        }
    }

    for r in openrouter_env_interns() {
        if names.insert(r.name.clone()) {
            out.push(r);
        }
    }

    for r in qwen_env_interns() {
        if names.insert(r.name.clone()) {
            out.push(r);
        }
    }

    out
}

fn resolve_one(cfg: &stynx_code_config::InternConfig) -> Option<ResolvedIntern> {
    let name = cfg.name.trim();
    if name.is_empty() { return None; }
    let provider = cfg.provider.trim().to_lowercase();

    let (default_base, default_key_env, provider_label) = match provider.as_str() {
        "deepseek" => ("https://api.deepseek.com/v1", "DEEPSEEK_API_KEY", "deepseek"),
        "openrouter" => ("https://openrouter.ai/api/v1", "OPENROUTER_API_KEY", "openrouter"),
        "openai" => ("https://api.openai.com/v1", "OPENAI_API_KEY", "openai"),
        "qwen" => (
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
            "QWEN_API_KEY",
            "qwen",
        ),
        "custom" => ("", "", "custom"),
        _ => {
            tracing::warn!(name = %name, provider = %provider, "unknown intern provider, skipping");
            return None;
        }
    };

    let base_url = cfg.base_url.clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_base.to_string());
    if base_url.is_empty() {
        tracing::warn!(name = %name, "custom intern needs base_url, skipping");
        return None;
    }

    let key_env = cfg.api_key_env.clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_key_env.to_string());
    let api_key = std::env::var(&key_env).ok().filter(|s| !s.trim().is_empty());
    let api_key = match api_key {
        Some(k) => k,
        None => {
            tracing::warn!(name = %name, env = %key_env, "intern missing api key in env, skipping");
            return None;
        }
    };

    let description = cfg.description.clone().unwrap_or_else(|| default_description(name, &cfg.model));

    Some(ResolvedIntern {
        name: name.to_string(),
        provider_label: provider_label.to_string(),
        description,
        base_url,
        api_key,
        model: cfg.model.clone(),
    })
}

fn legacy_deepseek_intern() -> Option<ResolvedIntern> {
    let api_key = std::env::var("DEEPSEEK_API_KEY").ok().filter(|s| !s.trim().is_empty())?;
    let base_url = std::env::var("DEEPSEEK_BASE_URL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
    let model = std::env::var("INTERN_MODEL")
        .or_else(|_| std::env::var("DEEPSEEK_MODEL"))
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "deepseek-v4-pro".to_string());
    Some(ResolvedIntern {
        name: "deepseek".to_string(),
        provider_label: "deepseek".to_string(),
        description: default_description("deepseek", &model),
        base_url,
        api_key,
        model,
    })
}

fn legacy_qwen_intern() -> Option<ResolvedIntern> {
    let api_key = std::env::var("QWEN_API_KEY").ok().filter(|s| !s.trim().is_empty())?;
    let base_url = std::env::var("QWEN_BASE_URL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://dashscope-intl.aliyuncs.com/compatible-mode/v1".to_string());
    let model = std::env::var("QWEN_MODEL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "qwen-plus".to_string());
    Some(ResolvedIntern {
        name: "qwen".to_string(),
        provider_label: "qwen".to_string(),
        description: default_description("qwen", &model),
        base_url,
        api_key,
        model,
    })
}

fn legacy_mimo_intern() -> Option<ResolvedIntern> {
    let api_key = std::env::var("MIMO_API_KEY").ok().filter(|s| !s.trim().is_empty())?;
    let base_url = std::env::var("MIMO_BASE_URL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://api.xiaomimimo.com/v1".to_string());
    let model = std::env::var("MIMO_MODEL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "mimo-v2.5-pro".to_string());
    Some(ResolvedIntern {
        name: "mimo".to_string(),
        provider_label: "custom".to_string(),
        description: default_description("mimo", &model),
        base_url,
        api_key,
        model,
    })
}

fn qwen_env_interns() -> Vec<ResolvedIntern> {
    let mut out: Vec<ResolvedIntern> = Vec::new();
    let api_key = match std::env::var("QWEN_API_KEY").ok().filter(|s| !s.trim().is_empty()) {
        Some(k) => k,
        None => return out,
    };
    let base_url = std::env::var("QWEN_BASE_URL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://dashscope-intl.aliyuncs.com/compatible-mode/v1".to_string());
    let spec = match std::env::var("QWEN_INTERNS").ok().filter(|s| !s.trim().is_empty()) {
        Some(s) => s,
        None => return out,
    };
    for entry in spec.split(',') {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        let (name, model) = match entry.split_once(':') {
            Some((n, m)) => (n.trim(), m.trim()),
            None => {
                tracing::warn!(entry = %entry, "QWEN_INTERNS entry must be name:model, skipping");
                continue;
            }
        };
        if name.is_empty() || model.is_empty() { continue; }
        out.push(ResolvedIntern {
            name: name.to_string(),
            provider_label: "qwen".to_string(),
            description: default_description(name, model),
            base_url: base_url.clone(),
            api_key: api_key.clone(),
            model: model.to_string(),
        });
    }
    out
}

fn openrouter_env_interns() -> Vec<ResolvedIntern> {
    let mut out: Vec<ResolvedIntern> = Vec::new();
    let api_key = match std::env::var("OPENROUTER_API_KEY").ok().filter(|s| !s.trim().is_empty()) {
        Some(k) => k,
        None => return out,
    };
    let base_url = std::env::var("OPENROUTER_BASE_URL")
        .ok().filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
    let spec = match std::env::var("OPENROUTER_INTERNS").ok().filter(|s| !s.trim().is_empty()) {
        Some(s) => s,
        None => return out,
    };
    for entry in spec.split(',') {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        let (name, model) = match entry.split_once(':') {
            Some((n, m)) => (n.trim(), m.trim()),
            None => {
                tracing::warn!(entry = %entry, "OPENROUTER_INTERNS entry must be name:model, skipping");
                continue;
            }
        };
        if name.is_empty() || model.is_empty() { continue; }
        out.push(ResolvedIntern {
            name: name.to_string(),
            provider_label: "openrouter".to_string(),
            description: default_description(name, model),
            base_url: base_url.clone(),
            api_key: api_key.clone(),
            model: model.to_string(),
        });
    }
    out
}

fn default_description(name: &str, model: &str) -> String {
    format!(
        "Hand off a focused, well-scoped subtask to the '{name}' intern (model: {model}). \
The intern has bash/read/file_write/file_edit/glob/grep available but cannot spawn further sub-agents. \
Use this for grunt work — boilerplate, mechanical refactors, gathering data, drafting code that you'll review. \
Provide explicit acceptance criteria. Returns the intern's summary + output."
    )
}

fn sanitize_tool_name(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.len() > 64 { out.truncate(64); }
    out
}
