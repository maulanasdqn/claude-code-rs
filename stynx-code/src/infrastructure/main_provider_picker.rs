use std::io::{self, Write};

use stynx_code_config::{InternConfig, Settings};

use super::interns::resolve_main_intern_candidates;

pub struct MainProviderChoice {
    pub kind: MainProviderKind,
    pub label: String,
}

pub enum MainProviderKind {
    Claude,
    Intern(InternConfig),
}

pub fn list_candidates(config: &Settings, claude_available: bool) -> Vec<MainProviderChoice> {
    let mut out = Vec::new();
    if claude_available {
        out.push(MainProviderChoice {
            kind: MainProviderKind::Claude,
            label: "claude".to_string(),
        });
    }
    for cfg in resolve_main_intern_candidates(config) {
        let label = cfg.name.clone();
        out.push(MainProviderChoice {
            kind: MainProviderKind::Intern(cfg),
            label,
        });
    }
    out
}

pub fn resolve_choice(
    cli_override: Option<&str>,
    config: &Settings,
    candidates: &[MainProviderChoice],
) -> Option<usize> {
    let configured = cli_override
        .map(str::to_string)
        .or_else(|| std::env::var("STYNX_MAIN_PROVIDER").ok())
        .or_else(|| config.main_provider.clone());
    if let Some(name) = configured.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return candidates.iter().position(|c| c.label.eq_ignore_ascii_case(name));
    }
    None
}

pub fn prompt_pick(candidates: &[MainProviderChoice]) -> Option<usize> {
    if candidates.len() < 2 { return Some(0); }
    println!("\n  Choose main agent provider:");
    for (i, c) in candidates.iter().enumerate() {
        println!("    {}. {}", i + 1, c.label);
    }
    print!("\n  Enter number [1]: ");
    io::stdout().flush().ok();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return Some(0);
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Some(0);
    }
    trimmed.parse::<usize>().ok()
        .and_then(|n| if n >= 1 && n <= candidates.len() { Some(n - 1) } else { None })
}

pub fn persist_choice(label: &str) -> std::io::Result<()> {
    let Some(home) = stynx_code_config::home_dir() else { return Ok(()); };
    let dir = home.join(".stynx");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("settings.json");
    let mut existing: serde_json::Value = if path.exists() {
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if let Some(obj) = existing.as_object_mut() {
        obj.insert("main_provider".into(), serde_json::Value::String(label.to_string()));
    }
    let body = serde_json::to_string_pretty(&existing).unwrap_or_default();
    std::fs::write(&path, body)
}
