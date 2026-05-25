use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(pub String);

impl PluginId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub enum PluginStatus {
    Installed,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: PathBuf,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginCapability {
    ProvidesTools,
    ProvidesCommands,
    ProvidesSkills,
}
