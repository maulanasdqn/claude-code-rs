use std::sync::{Arc, atomic::{AtomicBool, AtomicU8}};

use claude_rust_config::PermissionSettings;
use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionChecker, PermissionDecision, PermissionMode};
use serde_json::Value;

use crate::application::pattern_matcher::{parse_rule, rule_matches};
use crate::infrastructure::InteractivePermissionChecker;

pub struct ConfigAwarePermissionChecker {
    settings: PermissionSettings,
    interactive: InteractivePermissionChecker,
    mode: Arc<AtomicU8>,
}

impl ConfigAwarePermissionChecker {
    pub fn new(settings: PermissionSettings, mode: Arc<AtomicU8>) -> Self {
        Self { settings, interactive: InteractivePermissionChecker::new(), mode }
    }

    pub fn new_with_pause(settings: PermissionSettings, mode: Arc<AtomicU8>, paused: Arc<AtomicBool>) -> Self {
        Self { settings, interactive: InteractivePermissionChecker::with_flag(paused), mode }
    }

    pub fn pause_flag(&self) -> Arc<AtomicBool> {
        self.interactive.pause_flag()
    }
}

#[async_trait::async_trait]
impl PermissionChecker for ConfigAwarePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        let mode = PermissionMode::load(&self.mode);
        if mode == PermissionMode::AutoAccept || mode == PermissionMode::Bypass {
            return Ok(PermissionDecision::Allow);
        }
        if mode == PermissionMode::Plan {
            return Ok(PermissionDecision::Deny("plan mode: only read-only tools allowed".into()));
        }
        for rule_str in &self.settings.deny {
            if rule_matches(&parse_rule(rule_str), tool_name, input) {
                return Ok(PermissionDecision::Deny(format!("denied by config rule: {rule_str}")));
            }
        }
        for rule_str in &self.settings.allow {
            if rule_matches(&parse_rule(rule_str), tool_name, input) {
                return Ok(PermissionDecision::Allow);
            }
        }
        self.interactive.check(tool_name, input).await
    }
}
