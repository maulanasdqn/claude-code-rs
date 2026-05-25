use std::path::Path;

use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::plugin::{PluginId, PluginInfo, PluginStatus};
use stynx_code_errors::{AppError, AppResult};

#[async_trait]
pub trait PluginLoader {
    async fn load(&self, path: &Path) -> AppResult<PluginInfo>;
    async fn unload(&self, id: &PluginId) -> AppResult<()>;
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    description: String,
}

pub struct SubprocessPluginLoader;

#[async_trait]
impl PluginLoader for SubprocessPluginLoader {
    async fn load(&self, path: &Path) -> AppResult<PluginInfo> {
        let manifest_path = path.join("plugin.json");
        let contents = tokio::fs::read_to_string(&manifest_path).await.map_err(|e| {
            AppError::BadRequest(format!(
                "Failed to read plugin manifest at {}: {e}",
                manifest_path.display()
            ))
        })?;

        let manifest: PluginManifest = serde_json::from_str(&contents).map_err(|e| {
            AppError::BadRequest(format!("Invalid plugin manifest: {e}"))
        })?;

        if manifest.id.is_empty() {
            return Err(AppError::BadRequest("Plugin id must not be empty".into()));
        }
        if manifest.name.is_empty() {
            return Err(AppError::BadRequest("Plugin name must not be empty".into()));
        }
        if manifest.version.is_empty() {
            return Err(AppError::BadRequest("Plugin version must not be empty".into()));
        }

        Ok(PluginInfo {
            id: PluginId::new(manifest.id),
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            path: path.to_path_buf(),
            status: PluginStatus::Installed,
        })
    }

    async fn unload(&self, _id: &PluginId) -> AppResult<()> {
        Ok(())
    }
}
