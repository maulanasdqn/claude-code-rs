use super::{ConversationState, InputState, ModalState};

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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
