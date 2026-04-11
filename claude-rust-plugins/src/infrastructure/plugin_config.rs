use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::plugin::{PluginId, PluginInfo, PluginStatus};

#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry_point: String,
}

pub fn load_plugin_configs() -> Vec<PluginInfo> {
    let plugins_dir = match plugins_base_dir() {
        Some(p) => p,
        None => return vec![],
    };

    if !plugins_dir.exists() {
        return vec![];
    }

    let read_dir = match std::fs::read_dir(&plugins_dir) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut plugins = Vec::new();

    for entry in read_dir.flatten() {
        let plugin_dir = entry.path();
        if !plugin_dir.is_dir() {
            continue;
        }

        let manifest_path = plugin_dir.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        let contents = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let manifest: PluginManifest = match serde_json::from_str(&contents) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let id = entry.file_name().to_string_lossy().to_string();
        plugins.push(PluginInfo {
            id: PluginId::new(id),
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            path: plugin_dir,
            status: PluginStatus::Installed,
        });
    }

    plugins
}

fn plugins_base_dir() -> Option<PathBuf> {
    claude_rust_config::home_dir()
        .map(|h| h.join(".claude").join("plugins"))
}
