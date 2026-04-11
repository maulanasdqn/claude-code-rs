use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpServerEntry {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub fn load_mcp_configs(cwd: &str) -> Vec<(String, McpServerEntry)> {
    let home = claude_rust_config::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let paths = [
        format!("{cwd}/.mcp.json"),
        format!("{cwd}/.claude/mcp.json"),
        format!("{home}/.claude/mcp.json"),
    ];
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for path in &paths {
        if let Ok(text) = std::fs::read_to_string(path)
            && let Ok(cfg) = serde_json::from_str::<McpConfig>(&text) {
                for (name, entry) in cfg.mcp_servers {
                    if seen.insert(name.clone()) {
                        result.push((name, entry));
                    }
                }
            }
    }
    result
}
