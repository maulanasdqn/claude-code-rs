use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

/// Tool to list MCP resources.
///
/// TODO: Phase 4 — integrate with real MCP client.
pub struct ListMcpResourcesTool;

impl ListMcpResourcesTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for ListMcpResourcesTool {
    fn name(&self) -> &str {
        "list_mcp_resources"
    }

    fn description(&self) -> &str {
        "List available MCP resources, optionally filtered by server name."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "server_name": {
                    "type": "string",
                    "description": "Optional MCP server name to filter resources"
                }
            }
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_mcp(&self) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: false, is_list: true }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let server_name = input.get("server_name").and_then(|v| v.as_str());

        // TODO: Phase 4 — replace stub with real MCP resource listing
        tracing::info!(?server_name, "listing MCP resources (stub)");

        let result = json!({
            "resources": [],
            "server_name": server_name,
            "message": "MCP integration not yet available. Will be implemented in Phase 4."
        });

        Ok(result.to_string())
    }
}
