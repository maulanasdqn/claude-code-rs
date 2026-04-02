use std::collections::HashMap;

use tokio::process::Child;

use crate::domain::plugin::{PluginId, PluginInfo};
use claude_rust_errors::{AppError, AppResult};

pub struct PluginLifecycleManager {
    processes: HashMap<PluginId, Child>,
}

impl PluginLifecycleManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub async fn start(&mut self, info: &PluginInfo) -> AppResult<()> {
        if self.processes.contains_key(&info.id) {
            return Err(AppError::BadRequest(format!(
                "Plugin '{}' is already running",
                info.id
            )));
        }

        let entry_point = info.path.join("run");
        let child = tokio::process::Command::new(&entry_point)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!(
                    "Failed to spawn plugin '{}' at {}: {e}",
                    info.id,
                    entry_point.display()
                ))
            })?;

        self.processes.insert(info.id.clone(), child);
        Ok(())
    }

    pub async fn stop(&mut self, id: &PluginId) -> AppResult<()> {
        let child = self.processes.get_mut(id).ok_or_else(|| {
            AppError::BadRequest(format!("Plugin '{}' is not running", id))
        })?;

        child.kill().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to kill plugin '{}': {e}", id))
        })?;

        self.processes.remove(id);
        Ok(())
    }

    pub fn health_check(&mut self, id: &PluginId) -> bool {
        match self.processes.get_mut(id) {
            None => false,
            Some(child) => matches!(child.try_wait(), Ok(None)),
        }
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
