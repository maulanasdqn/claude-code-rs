use serde::Serialize;
use stynx_code_types::EngineEvent;

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UiEvent {
    TextDelta { text: String },
    ThinkingDelta { text: String },
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    ToolOutput { name: String, chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    Usage { input_tokens: u64, output_tokens: u64 },
    TurnComplete,
    Compacted { original_turns: u32 },
    ModeChanged { mode: String },
    HookOutput { source: String, output: String },
    SubAgentProgress { label: String, summary: String },
    SubAgentDone { label: String },
    RetryNotice { attempt: u32, max_attempts: u32, delay_ms: u64, message: String },
    Error { message: String },
    PermissionRequest { id: u64, tool_name: String, description: String },
    AskUserRequest { id: u64, question: String },
    WorkspaceMessageRequest { id: u64, target: String, task: String },
    Idle,
}

impl From<EngineEvent> for UiEvent {
    fn from(event: EngineEvent) -> Self {
        match event {
            EngineEvent::TextDelta(text) => UiEvent::TextDelta { text },
            EngineEvent::ThinkingDelta(text) => UiEvent::ThinkingDelta { text },
            EngineEvent::ToolStart { name, id } => UiEvent::ToolStart { name, id },
            EngineEvent::ToolInput { json_chunk } => UiEvent::ToolInput { json_chunk },
            EngineEvent::ToolOutput { name, chunk } => UiEvent::ToolOutput { name, chunk },
            EngineEvent::ToolResult { name, output, is_error } => {
                UiEvent::ToolResult { name, output, is_error }
            }
            EngineEvent::Usage { input_tokens, output_tokens } => {
                UiEvent::Usage { input_tokens, output_tokens }
            }
            EngineEvent::TurnComplete => UiEvent::TurnComplete,
            EngineEvent::Compacted { original_turns } => {
                UiEvent::Compacted { original_turns: original_turns as u32 }
            }
            EngineEvent::ModeChanged { mode } => UiEvent::ModeChanged { mode: mode.to_string() },
            EngineEvent::HookOutput { source, output } => UiEvent::HookOutput { source, output },
            EngineEvent::SubAgentProgress { label, summary } => {
                UiEvent::SubAgentProgress { label, summary }
            }
            EngineEvent::SubAgentDone { label } => UiEvent::SubAgentDone { label },
            EngineEvent::RetryNotice { attempt, max_attempts, delay_ms, message } => {
                UiEvent::RetryNotice { attempt, max_attempts, delay_ms, message }
            }
            EngineEvent::Error(message) => UiEvent::Error { message },
        }
    }
}
