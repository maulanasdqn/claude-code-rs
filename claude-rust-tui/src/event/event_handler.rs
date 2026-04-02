use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::state::{AppState, InputMode};

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
        if state.input.mode == InputMode::Normal {
            return Self::normal_mode_key(key, state);
        }
        Self::insert_mode_key(key, state)
    }

    fn insert_mode_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::Esc, _) => { state.input.mode = InputMode::Normal; UiAction::None }
            (KeyCode::Enter, _) => {
                let text = state.input.buffer.trim().to_string();
                if text.is_empty() { return UiAction::None; }
                state.input.push_history(text.clone());
                state.input.clear();
                UiAction::Submit(text)
            }
            (KeyCode::Backspace, _) => { state.input.delete_char(); UiAction::None }
            (KeyCode::Delete, _) => { state.input.delete_char_forward(); UiAction::None }
            (KeyCode::Left, KeyModifiers::ALT) => { state.input.move_word_left(); UiAction::None }
            (KeyCode::Right, KeyModifiers::ALT) => { state.input.move_word_right(); UiAction::None }
            (KeyCode::Left, _) => { state.input.move_cursor_left(); UiAction::None }
            (KeyCode::Right, _) => { state.input.move_cursor_right(); UiAction::None }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                state.input.cursor_pos = 0; UiAction::None
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                state.input.cursor_pos = state.input.buffer.len(); UiAction::None
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => { state.input.clear(); UiAction::None }
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => { state.input.move_word_left(); UiAction::None }
            (KeyCode::Up, _) => { state.input.history_prev(); UiAction::None }
            (KeyCode::Down, _) => { state.input.history_next(); UiAction::None }
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
                state.input.insert_char(c); UiAction::None
            }
            _ => UiAction::None,
        }
    }

    fn normal_mode_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::Char('i'), _) | (KeyCode::Char('a'), _) => {
                if key.code == KeyCode::Char('a') { state.input.move_cursor_right(); }
                state.input.mode = InputMode::Insert; UiAction::None
            }
            (KeyCode::Char('I'), _) => {
                state.input.cursor_pos = 0; state.input.mode = InputMode::Insert; UiAction::None
            }
            (KeyCode::Char('A'), _) => {
                state.input.cursor_pos = state.input.buffer.len(); state.input.mode = InputMode::Insert; UiAction::None
            }
            (KeyCode::Char('h'), _) | (KeyCode::Left, _) => { state.input.move_cursor_left(); UiAction::None }
            (KeyCode::Char('l'), _) | (KeyCode::Right, _) => { state.input.move_cursor_right(); UiAction::None }
            (KeyCode::Char('0'), _) => { state.input.cursor_pos = 0; UiAction::None }
            (KeyCode::Char('$'), _) => { state.input.cursor_pos = state.input.buffer.len(); UiAction::None }
            (KeyCode::Char('w'), _) => { state.input.move_word_right(); UiAction::None }
            (KeyCode::Char('b'), _) => { state.input.move_word_left(); UiAction::None }
            (KeyCode::Char('x'), _) => { state.input.delete_char_forward(); UiAction::None }
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => { state.input.history_prev(); UiAction::None }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => { state.input.history_next(); UiAction::None }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => { state.input.clear(); UiAction::None }
            (KeyCode::PageUp, _) => {
                state.conversation.auto_scroll = false;
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_sub(20);
                UiAction::None
            }
            (KeyCode::PageDown, _) => {
                state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_add(20);
                UiAction::None
            }
            (KeyCode::Enter, _) => {
                let text = state.input.buffer.trim().to_string();
                if text.is_empty() { return UiAction::None; }
                state.input.push_history(text.clone());
                state.input.clear();
                state.input.mode = InputMode::Insert;
                UiAction::Submit(text)
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
