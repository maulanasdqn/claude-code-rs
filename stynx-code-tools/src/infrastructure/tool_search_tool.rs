use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

pub struct ToolSearchTool {
    tool_catalog: Vec<(String, String)>,
}

impl ToolSearchTool {
    pub fn new(tool_catalog: Vec<(String, String)>) -> Self {
        Self { tool_catalog }
    }
}

#[async_trait::async_trait]
impl Tool for ToolSearchTool {
    fn name(&self) -> &str {
        "tool_search"
    }

    fn description(&self) -> &str {
        "Search available tools by name or description. Returns matching tool names with their descriptions."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to match against tool names and descriptions"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default 5)"
                }
            },
            "required": ["query"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }
    fn always_load(&self) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: true, is_read: false, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'query' field".into()))?;

        let max_results = input
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let query_lower = query.to_lowercase();

        tracing::info!(query, max_results, "searching tools");

        let matches: Vec<&(String, String)> = self
            .tool_catalog
            .iter()
            .filter(|(name, desc)| {
                name.to_lowercase().contains(&query_lower)
                    || desc.to_lowercase().contains(&query_lower)
            })
            .take(max_results)
            .collect();

        if matches.is_empty() {
            return Ok(format!("No tools matched query '{query}'."));
        }

        let mut result = format!("{} tool(s) matched:\n", matches.len());
        for (name, desc) in matches {
            result.push_str(&format!("\n- **{name}**: {desc}"));
        }

        Ok(result)
    }
}
