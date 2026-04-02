use async_trait::async_trait;
use claude_rust_errors::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: String,
}

#[async_trait]
pub trait PluginRegistry: Send + Sync {
    async fn discover(&self) -> Vec<PluginInfo>;
    async fn load(&self, name: &str) -> AppResult<()>;
    async fn unload(&self, name: &str);
}

pub struct StubPluginRegistry;

impl StubPluginRegistry {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PluginRegistry for StubPluginRegistry {
    async fn discover(&self) -> Vec<PluginInfo> {
        tracing::info!("plugin discovery not yet implemented");
        Vec::new()
    }

    async fn load(&self, name: &str) -> AppResult<()> {
        tracing::warn!(name, "plugin loading not yet implemented");
        Err(anyhow::anyhow!("plugin loading not yet implemented").into())
    }

    async fn unload(&self, name: &str) {
        tracing::warn!(name, "plugin unloading not yet implemented");
    }
}
