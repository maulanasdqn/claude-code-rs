/// Multi-agent conductor infrastructure.
///
/// The `AgentManager` tracks all spawned sub-agents and their results.
/// Three tools (`spawn_agent`, `wait_agent`, `list_agents`) are registered
/// in the main tool registry and exposed to the conductor engine so it can
/// orchestrate parallel worker agents.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;
use std::time::Instant;

use claude_rust_config::HooksConfig;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{
    Conversation, ContentBlock, Message, PermissionChecker, PermissionLevel, Role, Tool,
};
use claude_rust_tools::ToolRegistry;
use serde_json::{Value, json};

// ── Agent record ─────────────────────────────────────────────────────────────

pub enum AgentStatus {
    Running,
    Completed,
    Failed(String),
}

pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub task_preview: String,
    pub status: AgentStatus,
    pub result: Option<String>,
    pub started: Instant,
}

// ── Agent Manager ─────────────────────────────────────────────────────────────

pub struct AgentManager {
    records: Mutex<HashMap<String, AgentRecord>>,
    channels: Mutex<HashMap<String, tokio::sync::oneshot::Receiver<Result<String, String>>>>,
    counter: Mutex<u64>,
}

impl AgentManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            records: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            counter: Mutex::new(1),
        })
    }

    fn next_id(&self) -> String {
        let mut n = self.counter.lock().unwrap();
        let id = format!("agent-{n}");
        *n += 1;
        id
    }

    /// Register a new agent and return its ID plus the sender for the result.
    pub fn register(
        &self,
        name: String,
        task: String,
    ) -> (String, tokio::sync::oneshot::Sender<Result<String, String>>) {
        let id = self.next_id();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let preview = if task.len() > 80 {
            format!("{}…", &task[..79])
        } else {
            task.clone()
        };
        self.records.lock().unwrap().insert(id.clone(), AgentRecord {
            id: id.clone(),
            name,
            task_preview: preview,
            status: AgentStatus::Running,
            result: None,
            started: Instant::now(),
        });
        self.channels.lock().unwrap().insert(id.clone(), rx);
        (id, tx)
    }

    /// Wait for an agent to complete and return its output.
    pub async fn wait_for(&self, id: &str) -> Result<String, String> {
        let rx = self.channels.lock().unwrap().remove(id);
        let result = if let Some(rx) = rx {
            rx.await.unwrap_or_else(|_| Err("agent task was dropped".into()))
        } else {
            self.records.lock().unwrap()
                .get(id)
                .and_then(|r| r.result.clone())
                .map(Ok)
                .unwrap_or_else(|| Err(format!("agent '{id}' not found or already consumed")))
        };

        {
            let mut records = self.records.lock().unwrap();
            if let Some(rec) = records.get_mut(id) {
                match &result {
                    Ok(out) => {
                        rec.status = AgentStatus::Completed;
                        rec.result = Some(out.clone());
                    }
                    Err(e) => {
                        rec.status = AgentStatus::Failed(e.clone());
                    }
                }
            }
        }
        result
    }

    /// Snapshot of all agent statuses for display.
    pub fn snapshot(&self) -> Vec<AgentSnapshot> {
        self.records.lock().unwrap().values().map(|r| AgentSnapshot {
            id: r.id.clone(),
            name: r.name.clone(),
            task: r.task_preview.clone(),
            status: match &r.status {
                AgentStatus::Running => "running".into(),
                AgentStatus::Completed => "completed".into(),
                AgentStatus::Failed(e) => format!("failed: {e}"),
            },
            elapsed_s: r.started.elapsed().as_secs(),
        }).collect()
    }
}

#[derive(serde::Serialize)]
pub struct AgentSnapshot {
    pub id: String,
    pub name: String,
    pub task: String,
    pub status: String,
    pub elapsed_s: u64,
}

// ── Shared engine context (Clone + Send + Sync) ───────────────────────────────

#[derive(Clone)]
pub(crate) struct AgentCtx {
    pub provider: Arc<dyn claude_rust_types::Provider>,
    pub registry: Arc<ToolRegistry>,
    pub permission: Arc<dyn PermissionChecker>,
    pub mode: Arc<AtomicU8>,
    pub hooks: HooksConfig,
}

impl AgentCtx {
    pub async fn run(&self, system: &str, task: &str) -> AppResult<String> {
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
        conv.push(Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: task.to_string() }],
        });
        let output = Arc::new(Mutex::new(String::new()));
        let out_ref = output.clone();
        engine.run(conv, move |ev| {
            if let EngineEvent::TextDelta(t) = ev {
                out_ref.lock().unwrap().push_str(&t);
            }
        }).await?;
        Ok(output.lock().unwrap().clone())
    }
}

// ── spawn_agent tool ──────────────────────────────────────────────────────────

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
        provider: Arc<dyn claude_rust_types::Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
        hooks: HooksConfig,
        manager: Arc<AgentManager>,
    ) -> Self {
        Self {
            ctx: AgentCtx { provider, registry, permission, mode, hooks },
            manager,
        }
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
                "name":   { "type": "string", "description": "Short label, e.g. 'test-runner' or 'file-analyzer'" },
                "task":   { "type": "string", "description": "Full task description for the worker agent" },
                "system": { "type": "string", "description": "Optional custom system prompt override for this agent" }
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

// ── wait_agent tool ───────────────────────────────────────────────────────────

pub struct WaitAgentTool {
    manager: Arc<AgentManager>,
}

impl WaitAgentTool {
    pub fn new(manager: Arc<AgentManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for WaitAgentTool {
    fn name(&self) -> &str { "wait_agent" }

    fn description(&self) -> &str {
        "Wait for a spawned agent to finish and return its output. \
         Blocks until the agent completes. \
         You can call wait_agent on multiple agents in sequence to collect all results."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "The agent_id returned by spawn_agent" }
            },
            "required": ["agent_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let id = input["agent_id"].as_str()
            .ok_or_else(|| AppError::Tool("wait_agent: 'agent_id' is required".into()))?
            .to_string();

        match self.manager.wait_for(&id).await {
            Ok(output) => Ok(json!({
                "agent_id": id,
                "status": "completed",
                "output": output
            }).to_string()),
            Err(e) => Ok(json!({
                "agent_id": id,
                "status": "failed",
                "error": e
            }).to_string()),
        }
    }
}

// ── list_agents tool ──────────────────────────────────────────────────────────

pub struct ListAgentsTool {
    manager: Arc<AgentManager>,
}

impl ListAgentsTool {
    pub fn new(manager: Arc<AgentManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for ListAgentsTool {
    fn name(&self) -> &str { "list_agents" }

    fn description(&self) -> &str {
        "List all spawned agents with their id, name, status (running/completed/failed), \
         and elapsed seconds. Use this to check which agents are still running before waiting."
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }

    async fn execute(&self, _input: Value) -> AppResult<String> {
        let agents = self.manager.snapshot();
        Ok(serde_json::to_string(&agents).unwrap_or_else(|_| "[]".into()))
    }
}

// ── Conductor system prompt ───────────────────────────────────────────────────

pub const CONDUCTOR_SYSTEM: &str = "\
You are a conductor agent that orchestrates parallel worker agents to complete complex tasks.

## Your capabilities
- spawn_agent(name, task) — spawn a worker agent, returns agent_id immediately (non-blocking)
- wait_agent(agent_id)    — wait for a worker to finish and get its result
- list_agents()           — check status of all agents

## How to work
1. Break the task into independent subtasks
2. Spawn ALL independent agents in parallel (call spawn_agent multiple times before any wait_agent)
3. Wait for each agent with wait_agent to collect results
4. Synthesize all results into a final coherent answer

## Rules
- ALWAYS spawn parallel when tasks don't depend on each other — this is the whole point
- Keep task descriptions clear and self-contained so workers don't need context
- Synthesize results yourself — don't just dump raw agent outputs
- Be concise in your final summary

You also have access to regular tools (bash, read, glob, grep, etc.) for your own direct work.";
