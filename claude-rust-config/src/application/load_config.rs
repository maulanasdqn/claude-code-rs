use crate::domain::config::Settings;
use std::path::Path;

pub fn load_config() -> Settings {
    let global = load_file(&global_settings_path());
    let project = load_file(&project_settings_path());
    merge(global, project)
}

fn global_settings_path() -> std::path::PathBuf {
    let home = super::platform::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".claude").join("settings.json")
}

fn project_settings_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

fn load_file(path: &Path) -> Settings {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

fn merge(global: Settings, project: Settings) -> Settings {
    let mut allow = global.permissions.allow;
    allow.extend(project.permissions.allow);

    let mut deny = global.permissions.deny;
    deny.extend(project.permissions.deny);

    let mut pre_tool_use = global.hooks.pre_tool_use;
    pre_tool_use.extend(project.hooks.pre_tool_use);
    let mut post_tool_use = global.hooks.post_tool_use;
    post_tool_use.extend(project.hooks.post_tool_use);
    let mut stop = global.hooks.stop;
    stop.extend(project.hooks.stop);
    let mut session_start = global.hooks.session_start;
    session_start.extend(project.hooks.session_start);

    Settings {
        permissions: crate::domain::config::PermissionSettings { allow, deny },
        model: project.model.or(global.model),
        hooks: crate::domain::config::HooksConfig { pre_tool_use, post_tool_use, stop, session_start },
        max_turns: project.max_turns.or(global.max_turns),
        max_tokens: project.max_tokens.or(global.max_tokens),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{HooksConfig, PermissionSettings, Settings};

    #[test]
    fn test_merge_concatenates_permissions() {
        let global = Settings {
            permissions: PermissionSettings {
                allow: vec!["read".into(), "glob".into()],
                deny: vec!["bash(rm -rf *)".into()],
            },
            model: Some("global-model".into()),
            ..Default::default()
        };
        let project = Settings {
            permissions: PermissionSettings {
                allow: vec!["grep".into()],
                deny: vec![],
            },
            model: None,
            ..Default::default()
        };
        let merged = merge(global, project);
        assert_eq!(merged.permissions.allow, vec!["read", "glob", "grep"]);
        assert_eq!(merged.permissions.deny, vec!["bash(rm -rf *)"]);
        assert_eq!(merged.model, Some("global-model".into()));
    }

    #[test]
    fn test_merge_project_model_overrides() {
        let global = Settings {
            model: Some("global-model".into()),
            ..Default::default()
        };
        let project = Settings {
            model: Some("project-model".into()),
            ..Default::default()
        };
        let merged = merge(global, project);
        assert_eq!(merged.model, Some("project-model".into()));
    }
}
