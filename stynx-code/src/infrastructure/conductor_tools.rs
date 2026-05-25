use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_engine::EngineEvent;
use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionLevel, Role, SearchReadInfo, Tool,
};
use serde_json::{Value, json};

use super::conductor::{AgentCtx, AgentManager};

const WORKER_SYSTEM: &str = "\
You are a focused worker agent. Complete the assigned task efficiently \
and return a clear, concise, structured result. Use only the tools \
necessary. Do not over-explain.";

pub struct SpawnAgentTool {
    ctx: AgentCtx,
    manager: Arc<AgentManager>,
}

impl SpawnAgentTool {
    pub fn new(
        provider: Arc<dyn stynx_code_types::Provider>,
        registry: Arc<stynx_code_tools::ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
        hooks: stynx_code_config::HooksConfig,
        manager: Arc<AgentManager>,
    ) -> Self {
        Self { ctx: AgentCtx { provider, registry, permission, mode, hooks }, manager }
    }
}

#[async_trait::async_trait]
impl Tool for SpawnAgentTool {
    fn name(&self) -> &str { "spawn_agent" }
    fn description(&self) -> &str {
        "Spawn a parallel worker agent to handle an independent subtask. \
         Returns immediately with an agent_id — the agent runs in the background. \
         Use wait_agent(agent_id) to collect the result. \
         Spawn multiple agents before waiting to maximise parallelism."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name":   { "type": "string" },
                "task":   { "type": "string" },
                "system": { "type": "string" }
            },
            "required": ["name", "task"]
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let name = input["name"].as_str().unwrap_or("worker").to_string();
        let task = input["task"].as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Tool("spawn_agent: 'task' is required".into()))?
            .to_string();
        let system = input["system"].as_str().unwrap_or(WORKER_SYSTEM).to_string();
        let (id, tx) = self.manager.register(name.clone(), task.clone());
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let result = ctx.run(&system, &task).await.map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        Ok(json!({ "agent_id": id, "name": name, "status": "spawned" }).to_string())
    }
}

pub struct WaitAgentTool { manager: Arc<AgentManager> }

impl WaitAgentTool {
    pub fn new(manager: Arc<AgentManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for WaitAgentTool {
    fn name(&self) -> &str { "wait_agent" }
    fn description(&self) -> &str {
        "Wait for a spawned agent to finish and return its output."
    }
    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": { "agent_id": { "type": "string" } }, "required": ["agent_id"] })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    async fn execute(&self, input: Value) -> AppResult<String> {
        let id = input["agent_id"].as_str()
            .ok_or_else(|| AppError::Tool("wait_agent: 'agent_id' is required".into()))?
            .to_string();
        match self.manager.wait_for(&id).await {
            Ok(output) => Ok(json!({ "agent_id": id, "status": "completed", "output": output }).to_string()),
            Err(e) => Ok(json!({ "agent_id": id, "status": "failed", "error": e }).to_string()),
        }
    }
}

pub struct ListAgentsTool { manager: Arc<AgentManager> }

impl ListAgentsTool {
    pub fn new(manager: Arc<AgentManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for ListAgentsTool {
    fn name(&self) -> &str { "list_agents" }
    fn description(&self) -> &str {
        "List all spawned agents with their id, name, status, and elapsed seconds."
    }
    fn input_schema(&self) -> Value { json!({ "type": "object", "properties": {} }) }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }
    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: false, is_list: true }
    }
    async fn execute(&self, _input: Value) -> AppResult<String> {
        let agents = self.manager.snapshot();
        Ok(serde_json::to_string(&agents).unwrap_or_else(|_| "[]".into()))
    }
}

impl AgentCtx {
    pub async fn run(&self, system: &str, task: &str) -> AppResult<String> {
        use std::sync::Mutex;
        let engine = stynx_code_engine::QueryEngine::new(
            self.provider.clone(), self.registry.clone(), self.permission.clone(), self.mode.clone(), self.hooks.clone(),
        );
        let mut conv = Conversation { system: Some(system.to_string()), ..Default::default() };
        conv.push(Message { role: Role::User, content: vec![ContentBlock::Text { text: task.to_string() }] });
        let output = Arc::new(Mutex::new(String::new()));
        let out_ref = output.clone();
        engine.run(conv, move |ev| {
            if let EngineEvent::TextDelta(t) = ev { out_ref.lock().unwrap().push_str(&t); }
        }).await?;
        Ok(output.lock().unwrap().clone())
    }
}
