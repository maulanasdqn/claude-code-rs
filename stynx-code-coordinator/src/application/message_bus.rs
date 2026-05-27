use tokio::sync::broadcast;

use crate::domain::agent::AgentId;
use crate::domain::team::TeamMessage;

const CHANNEL_CAPACITY: usize = 256;

pub struct MessageBus {
    sender: broadcast::Sender<TeamMessage>,
}

impl MessageBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn send(&self, msg: TeamMessage) {

        let _ = self.sender.send(msg);
    }

    pub fn subscribe(&self, _agent_id: &AgentId) -> broadcast::Receiver<TeamMessage> {
        self.sender.subscribe()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
