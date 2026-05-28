use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_config::HooksConfig;
use stynx_code_errors::AppResult;
use stynx_code_types::{InterruptBehavior, PermissionChecker, PermissionLevel, Tool};
use stynx_code_tools::ToolRegistry;
use serde_json::{Value, json};

use super::sub_engine::SubEngine;

pub(super) const INTERN_SYSTEM: &str = "You are an intern engineer. A senior engineer (Claude) delegates tasks to you.\n\
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
        self.inner.run(&self.label, INTERN_SYSTEM, task).await
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

    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let task = input["task"].as_str().unwrap_or("").to_string();
        if task.trim().is_empty() {
            return Ok("[intern] no task provided".into());
        }
        tracing::info!(intern = %self.label, task_len = task.len(), "delegating to intern");
        let result = self.inner.run(&self.label, INTERN_SYSTEM, &task).await?;
        Ok(format!("[{label} intern]\n{result}", label = self.label))
    }
}
