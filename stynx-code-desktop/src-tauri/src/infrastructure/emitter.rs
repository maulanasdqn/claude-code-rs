use tauri::{AppHandle, Emitter};

use crate::domain::ui_event::UiEvent;

pub const ENGINE_EVENT: &str = "engine-event";

pub fn emit(app: &AppHandle, event: UiEvent) -> bool {
    app.emit(ENGINE_EVENT, event).is_ok()
}
