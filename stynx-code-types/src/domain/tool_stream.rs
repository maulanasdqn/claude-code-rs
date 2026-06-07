//! A task-local channel that lets a running tool stream incremental output
//! (e.g. bash stdout) up to the engine while it is still executing.
//!
//! The engine scopes [`TOOL_STREAM`] with a sender around each tool execution
//! and drains it concurrently, tagging chunks with the tool name. Tools call
//! [`emit`] — it is a no-op when no sink is in scope, so tools never depend on
//! the engine being present.

use tokio::sync::mpsc;

tokio::task_local! {
    pub static TOOL_STREAM: mpsc::UnboundedSender<String>;
}

/// Stream a chunk of output from the currently-executing tool. No-op when no
/// sink is installed (e.g. when run outside the engine).
pub fn emit(chunk: impl Into<String>) {
    let _ = TOOL_STREAM.try_with(|tx| tx.send(chunk.into()));
}
