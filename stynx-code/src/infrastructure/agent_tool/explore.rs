use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_config::HooksConfig;
use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionChecker, PermissionLevel, SearchReadInfo, Tool};
use stynx_code_tools::ToolRegistry;
use serde_json::{Value, json};

use super::sub_engine::SubEngine;

const EXPLORE_SYSTEM: &str = "You are a code exploration sub-agent. Analyze the codebase using read-only tools (read, glob, grep). Report findings clearly and concisely.";

pub struct ExploreAgentTool(SubEngine);

impl ExploreAgentTool {
    pub fn new(
        provider: Arc<dyn stynx_code_types::Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
        hooks: HooksConfig,
    ) -> Self {
        Self(SubEngine { provider, registry, permission, mode, hooks })
    }
}

#[async_trait::async_trait]
impl Tool for ExploreAgentTool {
    fn name(&self) -> &str { "explore" }

    fn description(&self) -> &str {
        "Spawn a read-only exploration sub-agent to analyze the codebase. Use this to search, read, and understand code without making changes. Returns the agent's findings."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "What to explore or analyze in the codebase"
                }
            },
            "required": ["task"]
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: true, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task = input["task"].as_str().unwrap_or("").to_string();
        self.0.run("explore", EXPLORE_SYSTEM, &task).await
    }
}
