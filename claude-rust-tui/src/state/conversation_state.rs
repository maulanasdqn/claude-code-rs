#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub role: String,
    pub content: String,
    pub tool_uses: Vec<DisplayToolUse>,
    pub is_streaming: bool,
}

#[derive(Debug, Clone)]
pub struct DisplayToolUse {
    pub name: String,
    pub status: ToolUseStatus,
    pub output_preview: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolUseStatus {
    Running,
    Completed,
    Error,
}

pub struct ConversationState {
    pub messages: Vec<DisplayMessage>,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
        }
    }
}

impl Default for ConversationState {
    fn default() -> Self {
        Self::new()
    }
}
