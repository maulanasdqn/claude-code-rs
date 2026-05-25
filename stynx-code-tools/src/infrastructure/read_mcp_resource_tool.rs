use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

/// Tool to read a specific MCP resource by URI.
///
/// TODO: Phase 4 — integrate with real MCP client.
pub struct ReadMcpResourceTool;

impl ReadMcpResourceTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for ReadMcpResourceTool {
    fn name(&self) -> &str {
        "read_mcp_resource"
    }

    fn description(&self) -> &str {
        "Read a specific MCP resource by its URI."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "uri": {
                    "type": "string",
                    "description": "The URI of the MCP resource to read"
                }
            },
            "required": ["uri"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_mcp(&self) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: true, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let uri = input
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'uri' field".into()))?;

        // TODO: Phase 4 — replace stub with real MCP resource reading
        tracing::info!(uri, "reading MCP resource (stub)");

        Ok(format!(
            "MCP resource not available: '{uri}'.\n\
             MCP integration will be implemented in Phase 4."
        ))
    }
}
