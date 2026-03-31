use claude_rust_config::PermissionSettings;
use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionChecker, PermissionDecision};
use serde_json::Value;

use crate::application::pattern_matcher::{parse_rule, rule_matches};
use crate::infrastructure::InteractivePermissionChecker;

/// A permission checker that consults config allowlists/denylists
/// before falling through to interactive prompting.
pub struct ConfigAwarePermissionChecker {
    settings: PermissionSettings,
    interactive: InteractivePermissionChecker,
}

impl ConfigAwarePermissionChecker {
    pub fn new(settings: PermissionSettings) -> Self {
        Self {
            settings,
            interactive: InteractivePermissionChecker::new(),
        }
    }
}

#[async_trait::async_trait]
impl PermissionChecker for ConfigAwarePermissionChecker {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision> {
        // 1. Check deny list first
        for rule_str in &self.settings.deny {
            let rule = parse_rule(rule_str);
            if rule_matches(&rule, tool_name, input) {
                return Ok(PermissionDecision::Deny(format!(
                    "denied by config rule: {rule_str}"
                )));
            }
        }

        // 2. Check allow list
        for rule_str in &self.settings.allow {
            let rule = parse_rule(rule_str);
            if rule_matches(&rule, tool_name, input) {
                return Ok(PermissionDecision::Allow);
            }
        }

        // 3. Fall through to interactive prompt
        self.interactive.check(tool_name, input).await
    }
}
