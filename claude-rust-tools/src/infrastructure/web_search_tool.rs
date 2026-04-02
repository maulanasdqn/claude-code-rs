use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

use super::web_search_client::search;

pub struct WebSearchTool;

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web using DuckDuckGo. Returns a list of results with titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "number",
                    "description": "Maximum number of results to return (default: 10)"
                }
            },
            "required": ["query"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }
    fn is_open_world(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: true, is_read: false, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let query = input
            .get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| AppError::Tool("missing 'query' field".into()))?;

        let max_results = input
            .get("max_results")
            .and_then(|m| m.as_u64())
            .unwrap_or(10) as usize;

        search(query, max_results).await
    }
}
