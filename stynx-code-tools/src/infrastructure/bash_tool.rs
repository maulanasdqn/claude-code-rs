use std::sync::Arc;
use std::time::Duration;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{InterruptBehavior, PermissionLevel, Tool};
use serde_json::{Value, json};

use super::persistent_shell::ShellRegistry;

const MAX_OUTPUT_BYTES: usize = 100_000;
const DEFAULT_TIMEOUT_SECS: u64 = 120;

pub struct BashTool {
    shells: Arc<ShellRegistry>,
}

impl BashTool {
    pub fn new() -> Self {
        Self { shells: Arc::new(ShellRegistry::new()) }
    }

    pub fn shells(&self) -> Arc<ShellRegistry> {
        self.shells.clone()
    }
}

impl Default for BashTool {
    fn default() -> Self { Self::new() }
}

#[async_trait::async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }

    fn description(&self) -> &str {
        "Run a bash command in a persistent shell session.\n\
\n\
Foreground (default): commands share a single long-lived bash process, so \
cwd, exported env vars, and shell aliases persist across calls. `cd subdir` \
in one call is visible to the next call.\n\
\n\
Long-running processes (dev servers, watchers, log tails): pass \
`background: true`. You get back a handle like `bg1`. Read its output with \
`{\"status\": \"bg1\"}` (returns only new output since last read), or \
`{\"status\": \"bg1\", \"full\": true}` for everything. Stop it with \
`{\"kill\": \"bg1\"}`. List all background processes with `{\"list\": true}`.\n\
\n\
Use `timeout` (seconds) to cap a foreground command — default 120s. \
If a foreground command times out, the shell may be in an unknown state; \
prefer `background: true` for anything that legitimately runs longer."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Bash command to run. Required unless using status/kill/list."
                },
                "background": {
                    "type": "boolean",
                    "description": "If true, spawn the command detached. Returns a handle like 'bg1' instead of waiting for completion."
                },
                "status": {
                    "type": "string",
                    "description": "Handle of a background process to read new output from."
                },
                "full": {
                    "type": "boolean",
                    "description": "When reading status, return all accumulated output instead of only new output since last read."
                },
                "kill": {
                    "type": "string",
                    "description": "Handle of a background process to terminate."
                },
                "list": {
                    "type": "boolean",
                    "description": "If true, list all background processes."
                },
                "timeout": {
                    "type": "integer",
                    "description": "Foreground command timeout in seconds (default 120)."
                }
            }
        })
    }

    fn permission_level(&self) -> PermissionLevel { PermissionLevel::Dangerous }

    fn interrupt_behavior(&self) -> InterruptBehavior { InterruptBehavior::Cancel }

    async fn execute(&self, input: Value) -> AppResult<String> {
        if input.get("list").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Ok(self.shells.list_background().await);
        }

        if let Some(handle) = input.get("kill").and_then(|v| v.as_str()) {
            return self.shells.kill_background(handle).await;
        }

        if let Some(handle) = input.get("status").and_then(|v| v.as_str()) {
            let full = input.get("full").and_then(|v| v.as_bool()).unwrap_or(false);
            return self.shells.read_background(handle, full).await;
        }

        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Tool("missing 'command' field".into()))?;

        let background = input.get("background").and_then(|v| v.as_bool()).unwrap_or(false);
        if background {
            tracing::info!(command, "starting background process");
            let handle = self.shells.run_background(command).await?;
            return Ok(format!(
                "started background process '{handle}'.\n\
read with {{\"status\":\"{handle}\"}}, stop with {{\"kill\":\"{handle}\"}}."
            ));
        }

        let timeout_secs = input
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        tracing::info!(command, timeout_secs, "executing bash (persistent)");

        let mut result = self
            .shells
            .run_sync(command, Some(Duration::from_secs(timeout_secs)))
            .await?;

        if result.len() > MAX_OUTPUT_BYTES {
            result.truncate(MAX_OUTPUT_BYTES);
            result.push_str("\n... (truncated)");
        }
        Ok(result)
    }
}
