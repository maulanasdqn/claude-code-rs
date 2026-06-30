use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};
use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use tokio::sync::{mpsc, oneshot};

pub struct WorkspaceRequest {
    pub target: String,
    pub task: String,
    pub responder: oneshot::Sender<String>,
}

#[derive(Clone)]
pub struct WorkspaceBridge {
    sender: mpsc::UnboundedSender<WorkspaceRequest>,
}

impl WorkspaceBridge {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<WorkspaceRequest>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    pub async fn send(&self, target: String, task: String) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        let request = WorkspaceRequest { target, task, responder: tx };
        if self.sender.send(request).is_err() {
            return None;
        }
        rx.await.ok()
    }
}

#[derive(Default)]
pub struct OptionalWorkspaceBridge(Mutex<Option<WorkspaceBridge>>);

impl OptionalWorkspaceBridge {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
    pub fn set(&self, bridge: WorkspaceBridge) {
        *self.0.lock().unwrap() = Some(bridge);
    }
    pub fn get(&self) -> Option<WorkspaceBridge> {
        self.0.lock().unwrap().clone()
    }
}

pub type SharedWorkspaceBridge = Arc<OptionalWorkspaceBridge>;

pub struct MessageWorkspaceTool {
    bridge: SharedWorkspaceBridge,
}

impl MessageWorkspaceTool {
    pub fn new(bridge: SharedWorkspaceBridge) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for MessageWorkspaceTool {
    fn name(&self) -> &str {
        "message_workspace"
    }

    fn description(&self) -> &str {
        "Send a task or question to the agent of another open workspace and wait for its reply. \
         Use this to coordinate cross-project changes (e.g. ask the frontend workspace to update \
         types after a backend API change). The target is the workspace folder name."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": { "type": "string", "description": "Target workspace folder name." },
                "task": { "type": "string", "description": "The task or question for that workspace's agent." }
            },
            "required": ["target", "task"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool {
        false
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let target = input.get("target").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let task = input.get("task").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        match self.bridge.get() {
            Some(bridge) => match bridge.send(target, task).await {
                Some(reply) => Ok(reply),
                None => Ok("(no other workspace is available to handle this)".to_string()),
            },
            None => Ok("(cross-workspace messaging is not connected)".to_string()),
        }
    }
}
