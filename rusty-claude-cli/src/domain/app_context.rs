use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8};

use claude_rust_config::Settings;
use claude_rust_engine::QueryEngine;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;

#[allow(dead_code)]
pub struct AppContext {
    pub engine: Arc<QueryEngine>,
    pub provider: Arc<AnthropicProvider>,
    pub config: Settings,
    pub mode_flag: Arc<AtomicU8>,
    pub pause_flag: Arc<AtomicBool>,
    pub permission: Arc<ConfigAwarePermissionChecker>,
    pub cwd: String,
}
