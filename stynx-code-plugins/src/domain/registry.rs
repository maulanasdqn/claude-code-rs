use crate::domain::plugin::{PluginCapability, PluginId, PluginInfo};
use stynx_code_errors::AppResult;

#[derive(Debug, Clone)]
pub struct PluginEntry {
    pub info: PluginInfo,
    pub capabilities: Vec<PluginCapability>,
}

pub trait PluginRegistryStore {
    fn register(&mut self, entry: PluginEntry) -> AppResult<()>;
    fn unregister(&mut self, id: &PluginId) -> AppResult<()>;
    fn get(&self, id: &PluginId) -> Option<&PluginEntry>;
    fn list(&self) -> Vec<PluginEntry>;
    fn find_by_capability(&self, cap: &PluginCapability) -> Vec<PluginEntry>;
}
