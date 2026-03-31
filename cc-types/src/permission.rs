use cc_errors::AppResult;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny(String),
}

#[async_trait::async_trait]
pub trait PermissionChecker: Send + Sync {
    async fn check(&self, tool_name: &str, input: &Value) -> AppResult<PermissionDecision>;
}

/// A checker that allows every tool call unconditionally.
pub struct AllowAll;

#[async_trait::async_trait]
impl PermissionChecker for AllowAll {
    async fn check(&self, _tool_name: &str, _input: &Value) -> AppResult<PermissionDecision> {
        Ok(PermissionDecision::Allow)
    }
}
