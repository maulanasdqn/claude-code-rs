use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_config::HooksConfig;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::AppResult;
use claude_rust_types::{
    Conversation, InterruptBehavior, Message, PermissionChecker, PermissionLevel, Role,
    SearchReadInfo, Tool,
};
use claude_rust_tools::ToolRegistry;
use serde_json::{Value, json};

const AGENT_SYSTEM: &str = "You are a specialized sub-agent. Complete the given task efficiently and concisely. Use tools as needed. Report results clearly.";

const EXPLORE_SYSTEM: &str = "You are a code exploration sub-agent. Analyze the codebase using read-only tools (read, glob, grep). Report findings clearly and concisely.";

struct SubEngine {
    provider: Arc<dyn claude_rust_types::Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    mode: Arc<AtomicU8>,
    hooks: HooksConfig,
}

impl SubEngine {
    async fn run(&self, system: &str, task: &str) -> AppResult<String> {
        let engine = QueryEngine::new(
            self.provider.clone(),
            self.registry.clone(),
            self.permission.clone(),
            self.mode.clone(),
            self.hooks.clone(),
        );
        let mut conv = Conversation {
            system: Some(system.to_string()),
            ..Default::default()
        };
        conv.push(Message { role: Role::User, content: vec![claude_rust_types::ContentBlock::Text { text: task.to_string() }] });

        let output = Arc::new(std::sync::Mutex::new(String::new()));
        let out_ref = output.clone();
        engine
            .run(conv, move |event| {
                if let EngineEvent::TextDelta(text) = event {
                    out_ref.lock().unwrap().push_str(&text);
                }
            })
            .await?;

        Ok(output.lock().unwrap().clone())
    }
}

pub struct AgentTool(SubEngine);

impl AgentTool {
    pub fn new(
        provider: Arc<dyn claude_rust_types::Provider>,
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

pub struct ExploreAgentTool(SubEngine);

impl ExploreAgentTool {
    pub fn new(
        provider: Arc<dyn claude_rust_types::Provider>,
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
        self.0.run(EXPLORE_SYSTEM, &task).await
    }
}
