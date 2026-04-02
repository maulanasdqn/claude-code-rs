pub mod domain;
pub mod application;
pub mod infrastructure;

pub use domain::plugin::{PluginId, PluginInfo, PluginStatus, PluginCapability};
pub use domain::registry::PluginRegistryStore;
pub use application::loader::PluginLoader;
pub use application::lifecycle::PluginLifecycleManager;
