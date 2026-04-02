use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::state::{AppState, ModalKind};

pub enum UiAction {
    Submit(String),
    Quit,
    None,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }

    pub fn handle(event: Event, state: &mut AppState) -> UiAction {
        let Event::Key(key) = event else { return UiAction::None; };
        if state.modal.active.is_some() { return Self::modal_key(key, state); }
        Self::normal_key(key, state)
    }

    fn normal_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::Enter, _) => {
                let text = state.input.buffer.trim().to_string();
                if text.is_empty() { return UiAction::None; }
                state.input.clear();
                UiAction::Submit(text)
            }
            (KeyCode::Backspace, _) => { state.input.delete_char(); UiAction::None }
            (KeyCode::Delete, _) => { state.input.delete_char(); UiAction::None }
            (KeyCode::Left, _) => { state.input.move_cursor_left(); UiAction::None }
            (KeyCode::Right, _) => { state.input.move_cursor_right(); UiAction::None }
            (KeyCode::Home, _) => { state.input.cursor_pos = 0; UiAction::None }
            (KeyCode::End, _) => { state.input.cursor_pos = state.input.buffer.len(); UiAction::None }
            (KeyCode::Up, _) => {
                state.conversation.auto_scroll = false;
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_sub(3);
                UiAction::None
            }
            (KeyCode::Down, _) => {
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_add(3);
                UiAction::None
            }
            (KeyCode::PageUp, _) => {
                state.conversation.auto_scroll = false;
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_sub(20);
                UiAction::None
            }
            (KeyCode::PageDown, _) => {
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_add(20);
                UiAction::None
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                state.input.insert_char(c);
                UiAction::None
            }
            _ => UiAction::None,
        }
    }

    fn modal_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        state.modal.active = None;
        let _ = key;
        UiAction::None
    }
}

impl Default for EventHandler {
    fn default() -> Self { Self }
}
