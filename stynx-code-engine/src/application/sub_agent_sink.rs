use stynx_code_types::EngineEvent;
use tokio::sync::mpsc;

tokio::task_local! {
    pub static SUB_AGENT_SINK: mpsc::UnboundedSender<EngineEvent>;
}

pub fn send(event: EngineEvent) {
    let _ = SUB_AGENT_SINK.try_with(|tx| tx.send(event));
}
