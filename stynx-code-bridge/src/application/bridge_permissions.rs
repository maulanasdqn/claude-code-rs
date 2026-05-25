use async_trait::async_trait;
use stynx_code_errors::AppResult;

#[async_trait]
pub trait BridgePermissionDelegate: Send + Sync {
    async fn request_permission(&self, tool_name: &str, input: &str) -> AppResult<bool>;
}

pub struct StubPermissionDelegate;

#[async_trait]
impl BridgePermissionDelegate for StubPermissionDelegate {
    async fn request_permission(&self, tool_name: &str, _input: &str) -> AppResult<bool> {
        tracing::debug!(tool_name, "auto-approving permission (stub delegate)");
        Ok(true)
    }
}
