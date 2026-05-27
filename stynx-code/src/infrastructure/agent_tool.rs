use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_config::HooksConfig;
use stynx_code_engine::{EngineEvent, QueryEngine};
use stynx_code_errors::AppResult;
use stynx_code_types::{
    Conversation, InterruptBehavior, Message, PermissionChecker, PermissionLevel, Role,
    SearchReadInfo, Tool,
};
use stynx_code_tools::ToolRegistry;
use serde_json::{Value, json};

const AGENT_SYSTEM: &str = "You are a specialized sub-agent. Complete the given task efficiently and concisely. Use tools as needed. Report results clearly.";

const EXPLORE_SYSTEM: &str = "You are a code exploration sub-agent. Analyze the codebase using read-only tools (read, glob, grep). Report findings clearly and concisely.";

const INTERN_SYSTEM: &str = "You are an intern engineer. A senior engineer (Claude) delegates tasks to you.\n\
\n\
RULES — violating any of these is a failure:\n\
1. ONLY report facts you observed via tool calls. Never invent, assume, or extrapolate.\n\
2. If you have not called a tool to verify something, you do not know it — do not say it.\n\
3. DO THE WORK. Reading context is not the deliverable. If asked to edit a file, call file_edit or file_write. If asked to find something, call grep or glob. If asked to run a command, call bash.\n\
4. Never hallucinate file paths, function names, crate names, counts, or any other concrete detail.\n\
5. If a task is ambiguous or impossible with available tools, say so immediately — do not guess.\n\
6. You cannot spawn sub-agents. You cannot call delegate_to_intern. Do not reference tools you do not have.\n\
7. No commentary, no filler, no apologies. Be direct and brief.\n\
\n\
AVAILABLE TOOLS: bash, read, file_write, file_edit, glob, grep\n\
\n\
OUTPUT FORMAT — you MUST end with this exact structure:\n\
  Summary: <one line: what you actually did>\n\
  Files changed:\n\
    - <absolute/path/to/file>\n\
  (write 'none' if nothing changed)\n\
  Output: <the deliverable or findings the senior asked for>\n\
\n\
Never end on a tool call. Always close with the summary block above.";

struct SubEngine {
    provider: Arc<dyn stynx_code_types::Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    mode: Arc<AtomicU8>,
    hooks: HooksConfig,
}

struct Activity {
    text: String,
    actions: Vec<String>,
    current_tool: Option<(String, String)>, // (name, input_json)
}

impl SubEngine {
    async fn run(&self, system: &str, task: &str) -> AppResult<String> {
        let sub_registry = Arc::new(self.registry.clone_excluding(&["agent", "explore"]));
        let engine = QueryEngine::new(
            self.provider.clone(),
            sub_registry,
            self.permission.clone(),
            self.mode.clone(),
            self.hooks.clone(),
        );
        let mut conv = Conversation {
            system: Some(system.to_string()),
            ..Default::default()
        };
        conv.push(Message { role: Role::User, content: vec![stynx_code_types::ContentBlock::Text { text: task.to_string() }] });

        let activity = std::sync::Arc::new(std::sync::Mutex::new(Activity {
            text: String::new(),
            actions: Vec::new(),
            current_tool: None,
        }));
        let act_ref = activity.clone();
        engine
            .run(conv, move |event| {
                let mut a = act_ref.lock().unwrap();
                match event {
                    EngineEvent::TextDelta(text) => a.text.push_str(&text),
                    EngineEvent::ToolStart { name, .. } => {
                        a.current_tool = Some((name, String::new()));
                    }
                    EngineEvent::ToolInput { json_chunk } => {
                        if let Some((_, buf)) = a.current_tool.as_mut() {
                            buf.push_str(&json_chunk);
                        }
                    }
                    EngineEvent::ToolResult { name, output, is_error } => {
                        let input = a
                            .current_tool
                            .take()
                            .filter(|(n, _)| n == &name)
                            .map(|(_, j)| j)
                            .unwrap_or_default();
                        let summary = summarize_action(&name, &input, &output, is_error);
                        a.actions.push(summary);
                    }
                    _ => {}
                }
            })
            .await?;

        let act = activity.lock().unwrap();
        let mut out = String::new();
        if !act.text.trim().is_empty() {
            out.push_str(act.text.trim());
            out.push('\n');
        }
        if !act.actions.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str("Actions taken:\n");
            for a in &act.actions {
                out.push_str("  • ");
                out.push_str(a);
                out.push('\n');
            }
        }
        if out.trim().is_empty() {
            out.push_str("(intern returned no output and took no actions)");
        }
        Ok(out)
    }
}

fn summarize_action(name: &str, input_json: &str, output: &str, is_error: bool) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_str(input_json).ok();
    let get_str = |k: &str| -> String {
        parsed
            .as_ref()
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let head = match name {
        "bash" => format!("bash $ {}", first_line_trunc(&get_str("command"), 100)),
        "read" => format!("read {}", get_str("file_path")),
        "file_write" => format!("write {}", get_str("file_path")),
        "file_edit" => format!("edit {}", get_str("file_path")),
        "glob" => format!("glob {}", get_str("pattern")),
        "grep" => {
            let p = get_str("pattern");
            let path = get_str("path");
            if path.is_empty() { format!("grep {p}") } else { format!("grep {p} in {path}") }
        }
        other => format!("{other}"),
    };
    let tail = if is_error {
        format!(" — ERROR: {}", first_line_trunc(output, 120))
    } else if name == "bash" || name == "file_write" || name == "file_edit" {
        let line_count = output.lines().filter(|l| !l.trim().is_empty()).count();
        if line_count > 0 { format!(" ({line_count} output lines)") } else { String::new() }
    } else {
        String::new()
    };
    format!("{head}{tail}")
}

fn first_line_trunc(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or("").trim();
    if line.chars().count() <= max {
        return line.to_string();
    }
    let mut out: String = line.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

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
        self.0.run(EXPLORE_SYSTEM, &task).await
    }
}

pub struct InternTool {
    inner: SubEngine,
    label: String,
    tool_name: String,
    description: String,
}

impl InternTool {
    pub fn new(
        provider: Arc<dyn stynx_code_types::Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
        hooks: HooksConfig,
        label: impl Into<String>,
        tool_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            inner: SubEngine { provider, registry, permission, mode, hooks },
            label: label.into(),
            tool_name: tool_name.into(),
            description: description.into(),
        }
    }

    pub fn label(&self) -> &str { &self.label }

    pub async fn run_task(&self, task: &str) -> AppResult<String> {
        self.inner.run(INTERN_SYSTEM, task).await
    }
}

#[async_trait::async_trait]
impl Tool for InternTool {
    fn name(&self) -> &str { &self.tool_name }

    fn description(&self) -> &str { &self.description }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Crisp task description, including acceptance criteria and any files / context the intern should look at."
                }
            },
            "required": ["task"]
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }

    fn interrupt_behavior(&self) -> InterruptBehavior { InterruptBehavior::Cancel }

    // Each intern is a separate API process — concurrent dispatch is safe.
    // This lets the senior call delegate_to_<a>, delegate_to_<b>,
    // delegate_to_<c> in a single turn and have them run in parallel.
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task = input["task"].as_str().unwrap_or("").to_string();
        if task.trim().is_empty() {
            return Ok("[intern] no task provided".into());
        }
        tracing::info!(intern = %self.label, task_len = task.len(), "delegating to intern");
        let result = self.inner.run(INTERN_SYSTEM, &task).await?;
        Ok(format!("[{label} intern]\n{result}", label = self.label))
    }
}

/// Fans a single task out to every configured intern in parallel and returns
/// their answers concatenated with per-intern headers.
pub struct AllInternsTool {
    interns: Vec<Arc<InternTool>>,
    description: String,
}

impl AllInternsTool {
    pub fn new(interns: Vec<Arc<InternTool>>) -> Self {
        let names: Vec<String> = interns.iter().map(|t| t.label().to_string()).collect();
        let description = if names.is_empty() {
            "Run the same task across all configured interns in parallel (no interns currently configured).".to_string()
        } else {
            format!(
                "Fan a single task out to ALL configured interns in parallel \
({names}) and return each intern's answer. Use this when you want multiple \
perspectives on the same problem, or to benchmark interns against each \
other. For ordinary delegation pick a specific delegate_to_<name> instead — \
this one is for explicit ensemble / comparison.",
                names = names.join(", "),
            )
        };
        Self { interns, description }
    }
}

#[async_trait::async_trait]
impl Tool for AllInternsTool {
    fn name(&self) -> &str { "delegate_to_all_interns" }
    fn description(&self) -> &str { &self.description }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Task to send to every intern simultaneously."
                }
            },
            "required": ["task"]
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }
    fn interrupt_behavior(&self) -> InterruptBehavior { InterruptBehavior::Cancel }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task = input["task"].as_str().unwrap_or("").trim().to_string();
        if task.is_empty() {
            return Ok("[interns] no task provided".into());
        }
        if self.interns.is_empty() {
            return Ok("[interns] no interns configured — set DEEPSEEK_API_KEY, OPENROUTER_API_KEY, QWEN_API_KEY in .env or add an `interns` array to settings.json.".into());
        }

        tracing::info!(intern_count = self.interns.len(), task_len = task.len(), "fanning out to all interns");

        let task_arc = Arc::new(task);
        let futures: Vec<_> = self.interns.iter().map(|tool| {
            let t = tool.clone();
            let task = task_arc.clone();
            tokio::spawn(async move {
                let label = t.label().to_string();
                let result = t.run_task(&task).await;
                (label, result)
            })
        }).collect();

        let mut sections: Vec<String> = Vec::with_capacity(futures.len());
        for handle in futures {
            match handle.await {
                Ok((label, Ok(output))) => {
                    sections.push(format!("─── {label} ───\n{output}"));
                }
                Ok((label, Err(e))) => {
                    sections.push(format!("─── {label} ───\n[failed: {e}]"));
                }
                Err(e) => {
                    sections.push(format!("─── ? ───\n[join error: {e}]"));
                }
            }
        }
        Ok(sections.join("\n\n"))
    }
}
