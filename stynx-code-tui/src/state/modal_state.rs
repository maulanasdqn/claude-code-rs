#[derive(Clone)]
pub struct DialogOption {
    pub value: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub footer: Option<String>,
    pub disabled: bool,
}

impl DialogOption {
    pub fn new(value: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            title: title.into(),
            description: None,
            category: None,
            footer: None,
            disabled: false,
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn with_category(mut self, c: impl Into<String>) -> Self {
        self.category = Some(c.into());
        self
    }

    pub fn with_footer(mut self, f: impl Into<String>) -> Self {
        self.footer = Some(f.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionChoice {
    Once,
    Always,
    Reject,
}

pub enum ModalKind {
    QuitConfirm,
    Permission {
        tool_name: String,
        description: String,
        choice: PermissionChoice,
    },
    Select {
        title: String,
        query: String,
        options: Vec<DialogOption>,
        selected: usize,
        current_value: Option<String>,
        kind: SelectKind,
        footer_hint: Option<String>,
    },
    Info {
        title: String,
        rows: Vec<(String, String)>,
    },
    Input {
        title: String,
        prompt: String,
        buffer: String,
        kind: InputKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    SessionRename,
    AskUserQuestion,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectKind {
    CommandPalette,
    ModelPicker,
    SessionList,
    Help,
    SkillPicker,
    FileMention,
    ThemePicker,
}

pub struct ModalState {
    pub active: Option<ModalKind>,
}

impl ModalState {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub fn open_select(
        &mut self,
        kind: SelectKind,
        title: impl Into<String>,
        options: Vec<DialogOption>,
        current_value: Option<String>,
        footer_hint: Option<String>,
    ) {
        let selected = current_value
            .as_ref()
            .and_then(|cv| options.iter().position(|o| &o.value == cv))
            .unwrap_or(0);
        self.active = Some(ModalKind::Select {
            title: title.into(),
            query: String::new(),
            options,
            selected,
            current_value,
            kind,
            footer_hint,
        });
    }

    pub fn open_quit_confirm(&mut self) {
        self.active = Some(ModalKind::QuitConfirm);
    }

    pub fn open_permission(&mut self, tool_name: impl Into<String>, description: impl Into<String>) {        self.active = Some(ModalKind::Permission {
            tool_name: tool_name.into(),
            description: description.into(),
            choice: PermissionChoice::Once,
        });
    }

    pub fn open_info(&mut self, title: impl Into<String>, rows: Vec<(String, String)>) {
        self.active = Some(ModalKind::Info { title: title.into(), rows });
    }

    pub fn open_input(
        &mut self,
        title: impl Into<String>,
        prompt: impl Into<String>,
        initial: impl Into<String>,
        kind: InputKind,
    ) {
        self.active = Some(ModalKind::Input {
            title: title.into(),
            prompt: prompt.into(),
            buffer: initial.into(),
            kind,
        });
    }

    pub fn close(&mut self) {
        self.active = None;
    }
}

impl Default for ModalState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn filter_options(opts: &[DialogOption], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return (0..opts.len()).collect();
    }
    opts.iter()
        .enumerate()
        .filter(|(_, o)| {
            o.title.to_lowercase().contains(&q)
                || o.description
                    .as_deref()
                    .map(|d| d.to_lowercase().contains(&q))
                    .unwrap_or(false)
                || o.category
                    .as_deref()
                    .map(|c| c.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .map(|(i, _)| i)
        .collect()
}
