pub enum ModalKind {
    Permission {
        tool_name: String,
        description: String,
    },
    ModelPicker {
        options: Vec<String>,
        selected: usize,
    },
    HistorySearch {
        query: String,
        results: Vec<String>,
        selected: usize,
    },
    Autocomplete {
        options: Vec<String>,
        selected: usize,
    },
}

pub struct ModalState {
    pub active: Option<ModalKind>,
}

impl ModalState {
    pub fn new() -> Self {
        Self { active: None }
    }
}

impl Default for ModalState {
    fn default() -> Self {
        Self::new()
    }
}
