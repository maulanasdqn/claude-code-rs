use std::sync::Arc;

use stynx_code_errors::AppResult;
use stynx_code_types::{InterruptBehavior, PermissionLevel, Tool};
use serde_json::{Value, json};

use super::intern::InternTool;

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
