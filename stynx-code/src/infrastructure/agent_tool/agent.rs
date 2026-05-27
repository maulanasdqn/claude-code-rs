use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_config::HooksConfig;
use stynx_code_errors::AppResult;
use stynx_code_types::{InterruptBehavior, PermissionChecker, PermissionLevel, Tool};
use stynx_code_tools::ToolRegistry;
use serde_json::{Value, json};

use super::sub_engine::SubEngine;

const AGENT_SYSTEM: &str = "You are a specialized sub-agent. Complete the given task efficiently and concisely. Use tools as needed. Report results clearly.";

pub struct AgentTool(SubEngine);

impl AgentTool {
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
impl Tool for AgentTool {
    fn name(&self) -> &str { "agent" }

    fn description(&self) -> &str {
        "Spawn a sub-agent to handle a complex, independent task. Use this to delegate research, analysis, or multi-step work that can run autonomously. Returns the agent's final response."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "The task description for the sub-agent"
                },
                "system_prompt": {
                    "type": "string",
                    "description": "Optional custom system prompt override for the sub-agent"
                }
            },
            "required": ["task"]
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }

    fn interrupt_behavior(&self) -> InterruptBehavior { InterruptBehavior::Cancel }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task = input["task"].as_str().unwrap_or("").to_string();
        let system = input["system_prompt"].as_str().unwrap_or(AGENT_SYSTEM);
        self.0.run(system, &task).await
    }
}
