use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::theme;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TuiState {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub sidebar_visible: Option<bool>,
    #[serde(default)]
    pub tool_details: Option<bool>,
    #[serde(default)]
    pub recent_models: Vec<String>,
}

fn state_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let path = PathBuf::from(home)
        .join(".stynx-code")
        .join("tui-state.json");
    Some(path)
}

pub fn load() -> TuiState {
    let Some(path) = state_path() else { return TuiState::default(); };
    let Ok(data) = std::fs::read_to_string(&path) else { return TuiState::default(); };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save(state: &TuiState) {
    let Some(path) = state_path() else { return; };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(data) = serde_json::to_string_pretty(state) else { return; };
    let _ = std::fs::write(&path, data);
}

pub fn apply_to(state: &mut AppState, persisted: &TuiState) {
    if let Some(id) = &persisted.theme {
        theme::set_theme(id);
    }
    if let Some(v) = persisted.sidebar_visible {
        state.sidebar.visible = v;
    }
    if let Some(v) = persisted.tool_details {
        state.tool_details = v;
    }
    if !persisted.recent_models.is_empty() {
        state.recent_models = persisted.recent_models.clone();
    }
}

pub fn snapshot(state: &AppState) -> TuiState {
    TuiState {
        theme: Some(theme::current().id.to_string()),
        sidebar_visible: Some(state.sidebar.visible),
        tool_details: Some(state.tool_details),
        recent_models: state.recent_models.clone(),
    }
}
