use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptChoice {
    AllowOnce,
    AllowAlways,
    Deny,
}

pub struct PromptRequest {
    pub tool_name: String,
    pub description: String,
    pub responder: oneshot::Sender<PromptChoice>,
}

#[derive(Clone)]
pub struct PromptBridge {
    sender: mpsc::UnboundedSender<PromptRequest>,
}

impl PromptBridge {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<PromptRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { sender: tx }, rx)
    }

    pub async fn request(
        &self,
        tool_name: String,
        description: String,
    ) -> Option<PromptChoice> {
        let (tx, rx) = oneshot::channel();
        let req = PromptRequest { tool_name, description, responder: tx };
        if self.sender.send(req).is_err() {
            return None;
        }
        rx.await.ok()
    }
}

#[derive(Default)]
pub struct OptionalBridge(Mutex<Option<PromptBridge>>);

impl OptionalBridge {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn set(&self, bridge: PromptBridge) {
        *self.0.lock().unwrap() = Some(bridge);
    }

    pub fn get(&self) -> Option<PromptBridge> {
        self.0.lock().unwrap().clone()
    }
}

pub type SharedBridge = Arc<OptionalBridge>;
