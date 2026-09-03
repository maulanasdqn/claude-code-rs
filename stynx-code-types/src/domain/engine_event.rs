use crate::PermissionMode;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    TextDelta(String),
    ThinkingDelta(String),
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    /// Incremental output streamed from a running tool (e.g. bash stdout).
    ToolOutput { name: String, chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    Usage { input_tokens: u64, output_tokens: u64 },
    TurnComplete,
    Compacted { original_turns: usize },
    ModeChanged { mode: PermissionMode },
    HookOutput { source: String, output: String },
    SubAgentProgress { label: String, summary: String },
    SubAgentDone { label: String },
    /// A transient provider failure (overloaded / rate-limited) is being retried
    /// after a backoff delay; the turn is still alive.
    RetryNotice { attempt: u32, max_attempts: u32, delay_ms: u64, message: String },
    Error(String),
}
