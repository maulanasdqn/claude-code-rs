use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use stynx_code_engine::{EngineEvent, QueryEngine};
use stynx_code_errors::AppError;
use stynx_code_types::Conversation;
use tokio::sync::mpsc;

pub(super) async fn run_engine_tui(
    engine: &Arc<QueryEngine>,
    conversation: Conversation,
    total_input: &Arc<AtomicU64>,
    total_output: &Arc<AtomicU64>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
) -> Result<Conversation, AppError> {
    let in_counter = total_input.clone();
    let out_counter = total_output.clone();
    let result = engine.run(conversation, move |event| {
        if let EngineEvent::Usage { input_tokens, output_tokens } = &event {
            if *input_tokens > 0 { in_counter.fetch_add(*input_tokens, Ordering::Relaxed); }
            if *output_tokens > 0 { out_counter.fetch_add(*output_tokens, Ordering::Relaxed); }
        }
        let _ = event_tx.send(event);
    });
    tokio::select! {
        r = result => r,
        _ = tokio::signal::ctrl_c() => Err(AppError::Interrupted),
    }
}
