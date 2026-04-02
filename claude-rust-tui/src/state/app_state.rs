use claude_rust_types::EngineEvent;
use super::{ConversationState, DisplayMessage, DisplayToolUse, InputState, ModalState, ToolUseStatus};

pub struct AppState {
    pub input: InputState,
    pub conversation: ConversationState,
    pub modal: ModalState,
    pub model_name: String,
    pub permission_mode: String,
    pub total_cost: f64,
    pub git_branch: Option<String>,
    pub is_streaming: bool,
    pub spinner_frame: usize,
    pub total_input: u64,
    pub total_output: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            conversation: ConversationState::new(),
            modal: ModalState::new(),
            model_name: String::from("claude-sonnet-4-20250514"),
            permission_mode: String::from("normal"),
            total_cost: 0.0,
            git_branch: None,
            is_streaming: false,
            spinner_frame: 0,
            total_input: 0,
            total_output: 0,
        }
    }

    pub fn push_user_message(&mut self, text: impl Into<String>) {
        self.conversation.messages.push(DisplayMessage {
            role: "user".to_string(),
            content: text.into(),
            tool_uses: Vec::new(),
            is_streaming: false,
        });
        self.conversation.auto_scroll = true;
    }

    pub fn push_system_message(&mut self, text: impl Into<String>) {
        self.conversation.messages.push(DisplayMessage {
            role: "system".to_string(),
            content: text.into(),
            tool_uses: Vec::new(),
            is_streaming: false,
        });
        self.conversation.auto_scroll = true;
    }

    pub fn apply_engine_event(&mut self, event: EngineEvent) {
        match event {
            EngineEvent::TextDelta(text) => {
                self.is_streaming = true;
                match self.conversation.messages.last_mut() {
                    Some(m) if m.role == "assistant" && m.is_streaming => m.content.push_str(&text),
                    _ => self.conversation.messages.push(DisplayMessage {
                        role: "assistant".to_string(), content: text,
                        tool_uses: Vec::new(), is_streaming: true,
                    }),
                }
            }
            EngineEvent::ThinkingDelta(_) => { self.is_streaming = true; }
            EngineEvent::ToolStart { name, .. } => {
                self.is_streaming = true;
                let tool = DisplayToolUse { name, status: ToolUseStatus::Running, output_preview: String::new() };
                match self.conversation.messages.last_mut().filter(|m| m.role == "assistant") {
                    Some(m) => m.tool_uses.push(tool),
                    None => self.conversation.messages.push(DisplayMessage {
                        role: "assistant".to_string(), content: String::new(),
                        tool_uses: vec![tool], is_streaming: true,
                    }),
                }
            }
            EngineEvent::ToolResult { name, output, is_error } => {
                if let Some(m) = self.conversation.messages.last_mut() {
                    if let Some(t) = m.tool_uses.iter_mut().rev()
                        .find(|t| t.name == name && t.status == ToolUseStatus::Running) {
                        t.status = if is_error { ToolUseStatus::Error } else { ToolUseStatus::Completed };
                        t.output_preview = output.lines().next().unwrap_or("").chars().take(80).collect();
                    }
                }
            }
            EngineEvent::TurnComplete => {
                self.is_streaming = false;
                if let Some(m) = self.conversation.messages.last_mut() { m.is_streaming = false; }
            }
            EngineEvent::Usage { input_tokens, output_tokens } => {
                if input_tokens > 0 { self.total_input += input_tokens; }
                if output_tokens > 0 { self.total_output += output_tokens; }
                self.total_cost = (self.total_input as f64 * 3.0 + self.total_output as f64 * 15.0) / 1_000_000.0;
            }
            EngineEvent::Error(e) => {
                self.is_streaming = false;
                self.conversation.messages.push(DisplayMessage {
                    role: "error".to_string(), content: e,
                    tool_uses: Vec::new(), is_streaming: false,
                });
            }
            _ => {}
        }
    }
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}
