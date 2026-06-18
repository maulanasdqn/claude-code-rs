use std::sync::Arc;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

use crate::intern_manager::{InternManager, InternStatus};

pub struct InternStatusTool {
    manager: Arc<InternManager>,
}

impl InternStatusTool {
    pub fn new(manager: Arc<InternManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for InternStatusTool {
    fn name(&self) -> &str { "intern_status" }
    fn description(&self) -> &str {
        "Peek at a background intern delegation. Pass the handle returned by delegate_to_<intern> with background=true. \
         Returns {handle, intern, status, elapsed_s, last_action, task}. Status is one of: running, completed, failed, killed, timed_out. \
         Omit the handle to list all known interns."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "handle": { "type": "string", "description": "intern handle, e.g. intern-3. omit to list all." }
            }
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }
    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: false, is_list: true }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        if let Some(handle) = input.get("handle").and_then(|v| v.as_str()) {
            let snap = self.manager.status_snapshot(handle)
                .ok_or_else(|| AppError::Tool(format!("intern '{handle}' not found")))?;
            let status = match &snap.status {
                InternStatus::Running => "running".to_string(),
                InternStatus::Completed => "completed".to_string(),
                InternStatus::Failed(e) => format!("failed: {e}"),
                InternStatus::Killed => "killed".to_string(),
                InternStatus::TimedOut => "timed_out".to_string(),
            };
            Ok(json!({
                "handle": snap.id,
                "intern": snap.intern,
                "task": snap.task_preview,
                "status": status,
                "elapsed_s": snap.started.elapsed().as_secs(),
                "last_action": snap.last_action,
            }).to_string())
        } else {
            let snaps = self.manager.snapshot();
            Ok(serde_json::to_string(&snaps).unwrap_or_else(|_| "[]".into()))
        }
    }
}

pub struct InternKillTool { manager: Arc<InternManager> }

impl InternKillTool {
    pub fn new(manager: Arc<InternManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for InternKillTool {
    fn name(&self) -> &str { "intern_kill" }
    fn description(&self) -> &str {
        "Abort a running background intern delegation. Use when the intern is stuck, looping, or going wild. \
         The task is dropped and any partial work is whatever the intern committed before the abort."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "handle": { "type": "string" } },
            "required": ["handle"]
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let handle = input.get("handle").and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("intern_kill: 'handle' is required".into()))?;
        let killed = self.manager.kill(handle);
        if killed {
            Ok(json!({"handle": handle, "status": "killed"}).to_string())
        } else {
            Ok(json!({"handle": handle, "status": "not_running", "note": "already finished, killed, or unknown handle"}).to_string())
        }
    }
}

pub struct InternWaitTool { manager: Arc<InternManager> }

impl InternWaitTool {
    pub fn new(manager: Arc<InternManager>) -> Self { Self { manager } }
}

#[async_trait::async_trait]
impl Tool for InternWaitTool {
    fn name(&self) -> &str { "intern_wait" }
    fn description(&self) -> &str {
        "Block until a background intern finishes and return its output. Optionally pass max_wait_secs (default 300) — \
         if the intern is still running after that, returns a timeout (the intern keeps running; call intern_kill to abort it). \
         Returns {handle, status, output} on success."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "handle": { "type": "string" },
                "max_wait_secs": { "type": "number" }
            },
            "required": ["handle"]
        })
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let handle = input.get("handle").and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("intern_wait: 'handle' is required".into()))?
            .to_string();
        let max_wait_secs = input.get("max_wait_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(300);
        match self.manager.wait_for(&handle, max_wait_secs).await {
            Ok(output) => Ok(json!({"handle": handle, "status": "completed", "output": output}).to_string()),
            Err(e) if e.starts_with("wait timed out") => {
                Ok(json!({
                    "handle": handle,
                    "status": "still_running",
                    "note": format!("intern still running after {max_wait_secs}s — call intern_status, intern_wait again with a larger budget, or intern_kill to abort."),
                }).to_string())
            }
            Err(e) => Ok(json!({"handle": handle, "status": "failed", "error": e}).to_string()),
        }
    }
}
