use std::sync::Arc;

use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::Value;
use tokio::sync::Mutex;

use super::mcp_client::McpClient;

pub struct McpDynamicTool {
    pub tool_name: String,
    pub tool_description: String,
    pub tool_schema: Value,
    pub client: Arc<Mutex<McpClient>>,
    pub raw_name: String,
}

#[async_trait::async_trait]
impl Tool for McpDynamicTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn input_schema(&self) -> Value {
        self.tool_schema.clone()
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let mut client = self.client.lock().await;
        client.call_tool(&self.raw_name, input).await
    }
}
