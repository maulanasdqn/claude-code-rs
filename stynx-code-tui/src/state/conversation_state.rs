#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub role: String,
    pub content: String,
    pub thinking: String,
    pub tool_uses: Vec<DisplayToolUse>,
    pub is_streaming: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DisplayToolUse {
    pub name: String,
    pub status: ToolUseStatus,
    pub output_preview: String,
    pub input_json: String,
    pub input_summary: String,
    pub output_excerpt: Vec<String>,
    pub diff: Vec<DiffLine>,
    pub sub_progress: Vec<String>,
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
    pub total_lines: usize,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            total_lines: 0,
        }
    }
}

impl Default for ConversationState {
    fn default() -> Self {
        Self::new()
    }
}
