#[derive(Debug, Clone)]
pub enum EngineEvent {
    TextDelta(String),
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    TurnComplete,
    Error(String),
}
