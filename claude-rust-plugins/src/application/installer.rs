use std::path::Path;

use crate::application::loader::{PluginLoader, SubprocessPluginLoader};
use crate::domain::plugin::{PluginId, PluginInfo};
use claude_rust_errors::{AppError, AppResult};

pub struct PluginInstaller;

impl PluginInstaller {
    pub fn new() -> Self {
        Self
    }

    pub async fn install_from_path(&self, src: &Path, dest: &Path) -> AppResult<PluginInfo> {
        if !src.exists() {
            return Err(AppError::BadRequest(format!(
                "Source plugin path does not exist: {}",
                src.display()
            )));
        }

        tokio::fs::create_dir_all(dest).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "Failed to create plugin destination {}: {e}",
                dest.display()
            ))
        })?;

        copy_dir_all(src, dest).await?;

        let loader = SubprocessPluginLoader;
        loader.load(dest).await
    }

    pub async fn uninstall(&self, id: &PluginId) -> AppResult<()> {
        let plugins_dir = plugins_base_dir()?;
        let plugin_path = plugins_dir.join(id.as_str());

        if !plugin_path.exists() {
            return Err(AppError::BadRequest(format!(
                "Plugin '{}' is not installed",
                id
            )));
        }

        tokio::fs::remove_dir_all(&plugin_path).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!(
                "Failed to remove plugin directory {}: {e}",
                plugin_path.display()
            ))
        })?;

        Ok(())
    }
}

impl Default for PluginInstaller {
    fn default() -> Self {
        Self::new()
    }
}

fn plugins_base_dir() -> AppResult<std::path::PathBuf> {
    let home = dirs_home()?;
    Ok(home.join(".claude").join("plugins"))
}

fn dirs_home() -> AppResult<std::path::PathBuf> {
    claude_rust_config::home_dir()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("cannot determine home directory")))
}

async fn copy_dir_all(src: &Path, dst: &Path) -> AppResult<()> {
    let mut entries = tokio::fs::read_dir(src).await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to read dir {}: {e}", src.display()))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to read dir entry: {e}"))
    })? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to get file type: {e}"))
        })?;

        if file_type.is_dir() {
            tokio::fs::create_dir_all(&dst_path).await.map_err(|e| {
                AppError::Internal(anyhow::anyhow!(
                    "Failed to create dir {}: {e}",
                    dst_path.display()
                ))
            })?;
            Box::pin(copy_dir_all(&src_path, &dst_path)).await?;
        } else {
            tokio::fs::copy(&src_path, &dst_path).await.map_err(|e| {
                AppError::Internal(anyhow::anyhow!(
                    "Failed to copy {} -> {}: {e}",
                    src_path.display(),
                    dst_path.display()
                ))
            })?;
        }
    }

    Ok(())
}
