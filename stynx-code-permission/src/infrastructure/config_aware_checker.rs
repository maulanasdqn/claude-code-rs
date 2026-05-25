use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU8}};

use stynx_code_config::PermissionSettings;
use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionChecker, PermissionDecision, PermissionMode};
use serde_json::Value;

use crate::application::pattern_matcher::{parse_rule, rule_matches};
use crate::infrastructure::InteractivePermissionChecker;
use crate::infrastructure::prompt_bridge::PromptBridge;

pub struct ConfigAwarePermissionChecker {
    settings: PermissionSettings,
    interactive: InteractivePermissionChecker,
    mode: Arc<AtomicU8>,
    /// Temporary allow rules from skill `allowed-tools` frontmatter.
    skill_allow_rules: RwLock<Vec<String>>,
}

impl ConfigAwarePermissionChecker {
    pub fn new(settings: PermissionSettings, mode: Arc<AtomicU8>) -> Self {
        Self { settings, interactive: InteractivePermissionChecker::new(), mode, skill_allow_rules: RwLock::new(Vec::new()) }
    }

    pub fn new_with_pause(settings: PermissionSettings, mode: Arc<AtomicU8>, paused: Arc<AtomicBool>) -> Self {
        Self { settings, interactive: InteractivePermissionChecker::with_flag(paused), mode, skill_allow_rules: RwLock::new(Vec::new()) }
    }

    pub fn pause_flag(&self) -> Arc<AtomicBool> {
        self.interactive.pause_flag()
    }

    pub fn install_prompt_bridge(&self, bridge: PromptBridge) {
        self.interactive.bridge_handle().set(bridge);
    }

    /// Set skill-specific allow rules (from `allowed-tools` frontmatter).
    /// These are checked in addition to config-level allow rules.
    pub fn set_skill_allow_rules(&self, rules: Vec<String>) {
        *self.skill_allow_rules.write().unwrap() = rules;
    }

    /// Clear skill-specific allow rules (call after skill execution completes).
    pub fn clear_skill_allow_rules(&self) {
        self.skill_allow_rules.write().unwrap().clear();
    }
}

#[async_trait::async_trait]
impl PermissionChecker for ConfigAwarePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        let mode = PermissionMode::load(&self.mode);
        for rule_str in &self.settings.deny {
            if rule_matches(&parse_rule(rule_str), tool_name, input) {
                return Ok(PermissionDecision::Deny(format!("denied by config rule: {rule_str}")));
            }
        }
        if mode == PermissionMode::AutoAccept || mode == PermissionMode::Bypass {
            return Ok(PermissionDecision::Allow);
        }
        if mode == PermissionMode::Plan {
            return Ok(PermissionDecision::Deny("plan mode: only read-only tools allowed".into()));
        }
        for rule_str in &self.settings.allow {
            if rule_matches(&parse_rule(rule_str), tool_name, input) {
                return Ok(PermissionDecision::Allow);
            }
        }
        // Check skill-specific allow rules
        {
            let skill_rules = self.skill_allow_rules.read().unwrap();
            for rule_str in skill_rules.iter() {
                if rule_matches(&parse_rule(rule_str), tool_name, input) {
                    return Ok(PermissionDecision::Allow);
                }
            }
        }
        self.interactive.check(tool_name, input).await
    }
}
