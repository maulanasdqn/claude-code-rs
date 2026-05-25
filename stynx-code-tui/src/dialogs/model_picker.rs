use crate::state::{AppState, DialogOption, SelectKind};

struct ModelEntry {
    id: &'static str,
    name: &'static str,
    provider: &'static str,
    note: Option<&'static str>,
}

const MODELS: &[ModelEntry] = &[
    ModelEntry {
        id: "claude-opus-4-7",
        name: "Claude Opus 4.7",
        provider: "Anthropic",
        note: Some("most capable"),
    },
    ModelEntry {
        id: "claude-opus-4-6",
        name: "Claude Opus 4.6",
        provider: "Anthropic",
        note: None,
    },
    ModelEntry {
        id: "claude-sonnet-4-6",
        name: "Claude Sonnet 4.6",
        provider: "Anthropic",
        note: Some("balanced"),
    },
    ModelEntry {
        id: "claude-haiku-4-5-20251001",
        name: "Claude Haiku 4.5",
        provider: "Anthropic",
        note: Some("fast"),
    },
    ModelEntry {
        id: "claude-sonnet-4-20250514",
        name: "Claude Sonnet 4",
        provider: "Anthropic",
        note: None,
    },
    ModelEntry {
        id: "claude-3-7-sonnet-20250219",
        name: "Claude Sonnet 3.7",
        provider: "Anthropic",
        note: None,
    },
];

pub fn open_model_picker(state: &mut AppState) {
    let current = state.model_name.clone();
    let options: Vec<DialogOption> = MODELS
        .iter()
        .map(|m| {
            let mut opt = DialogOption::new(m.id, m.name).with_category(m.provider);
            if let Some(note) = m.note {
                opt = opt.with_footer(note);
            }
            opt
        })
        .collect();
    state.modal.open_select(
        SelectKind::ModelPicker,
        "Select model",
        options,
        Some(current),
        Some("• current".to_string()),
    );
}
