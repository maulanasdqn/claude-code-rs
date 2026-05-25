use crate::PermissionMode;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    TextDelta(String),
    ThinkingDelta(String),
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    Usage { input_tokens: u64, output_tokens: u64 },
    TurnComplete,
    Compacted { original_turns: usize },
    ModeChanged { mode: PermissionMode },
    HookOutput { source: String, output: String },
    Error(String),
}
