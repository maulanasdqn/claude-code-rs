pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::ToolRegistry;
pub use infrastructure::{
    AskUserTool, BashTool, EnterPlanModeTool, ExitPlanModeTool, FileEditTool, FileWriteTool,
    GlobTool, GrepTool, ReadTool, TodoReadTool, TodoWriteTool, WebFetchTool, WebSearchTool,
};

pub async fn load_mcp_tools(cwd: &str) -> Vec<std::sync::Arc<dyn claude_rust_types::Tool>> {
    use infrastructure::mcp_client::McpClient;
    use infrastructure::mcp_config::load_mcp_configs;
    use infrastructure::mcp_tool::McpDynamicTool;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let mut tools: Vec<Arc<dyn claude_rust_types::Tool>> = Vec::new();
    for (server_name, entry) in load_mcp_configs(cwd) {
        let mut client = match McpClient::start(&server_name, &entry).await {
            Ok(c) => c,
            Err(e) => { tracing::warn!("MCP '{server_name}' start failed: {e}"); continue; }
        };
        let tool_list = match client.list_tools().await {
            Ok(t) => t,
            Err(e) => { tracing::warn!("MCP '{server_name}' list_tools failed: {e}"); continue; }
        };
        let shared = Arc::new(Mutex::new(client));
        for (raw_name, description, schema) in tool_list {
            let tool_name = format!("mcp__{server_name}__{raw_name}");
            tools.push(Arc::new(McpDynamicTool {
                tool_name,
                tool_description: description,
                tool_schema: schema,
                client: shared.clone(),
                raw_name,
            }));
        }
        tracing::info!("MCP '{server_name}' loaded {} tools", tools.len());
    }
    tools
}
