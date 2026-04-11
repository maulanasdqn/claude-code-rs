use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::state::{AppState, InputMode};

pub enum UiAction {
    Submit(String),
    CyclePermissionMode,
    Quit,
    None,
}

const COMMANDS: &[&str] = &[
    "/add", "/commit", "/config", "/copy", "/effort", "/exit",
    "/export", "/fast", "/files", "/help", "/memory", "/mode",
    "/model", "/permissions", "/plan", "/quit", "/rewind",
    "/review", "/skills", "/think", "/usage", "/version",
];

fn update_suggestion(state: &mut AppState) {
    let buf = &state.input.buffer;
    if buf.starts_with('/') && !buf.contains(' ') {
        let matched: Vec<&str> = COMMANDS.iter().copied()
            .filter(|c| c.starts_with(buf.as_str()) && *c != buf.as_str())
            .collect();
        state.input.suggestion = if matched.len() == 1 {
            matched[0][buf.len()..].to_string()
        } else {
            String::new()
        };
    } else {
        state.input.suggestion = String::new();
    }
}

fn scroll_up(state: &mut AppState, n: usize) {
    state.conversation.auto_scroll = false;
    state.conversation.scroll_offset = state.conversation.scroll_offset.saturating_sub(n);
}

fn scroll_down(state: &mut AppState, n: usize) {
    let next = state.conversation.scroll_offset.saturating_add(n);
    if next >= state.conversation.total_lines {
        state.conversation.auto_scroll = true;
    } else {
        state.conversation.scroll_offset = next;
    }
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }

    pub fn handle(event: Event, state: &mut AppState) -> UiAction {
        let Event::Key(key) = event else { return UiAction::None; };
        if key.kind != KeyEventKind::Press { return UiAction::None; }
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
            (KeyCode::BackTab, _) => UiAction::CyclePermissionMode,
            (KeyCode::Tab, _) => { state.input.complete_suggestion(); UiAction::None }
            (KeyCode::Enter, _) => {
                let text = state.input.buffer.trim().to_string();
                if text.is_empty() { return UiAction::None; }
                state.input.push_history(text.clone());
                state.input.clear();
                UiAction::Submit(text)
            }
            (KeyCode::Backspace, _) => {
                state.input.delete_char(); update_suggestion(state); UiAction::None
            }
            (KeyCode::Delete, _) => {
                state.input.delete_char_forward(); update_suggestion(state); UiAction::None
            }
            (KeyCode::Left, KeyModifiers::ALT) => { state.input.move_word_left(); UiAction::None }
            (KeyCode::Right, KeyModifiers::ALT) => { state.input.move_word_right(); UiAction::None }
            (KeyCode::Left, _) => { state.input.move_cursor_left(); UiAction::None }
            (KeyCode::Right, _) => {
                if state.input.cursor_pos == state.input.buffer.len() && !state.input.suggestion.is_empty() {
                    state.input.complete_suggestion();
                } else {
                    state.input.move_cursor_right();
                }
                UiAction::None
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                state.input.cursor_pos = 0; UiAction::None
            }
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                state.input.cursor_pos = state.input.buffer.len(); UiAction::None
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                state.input.clear(); update_suggestion(state); UiAction::None
            }
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                state.input.move_word_left(); update_suggestion(state); UiAction::None
            }
            (KeyCode::Up, _) => {
                if state.input.buffer.is_empty() {
                    scroll_up(state, 3);
                } else {
                    state.input.history_prev();
                }
                UiAction::None
            }
            (KeyCode::Down, _) => {
                if state.input.buffer.is_empty() {
                    scroll_down(state, 3);
                } else {
                    state.input.history_next();
                }
                UiAction::None
            }
            (KeyCode::PageUp, _) => { scroll_up(state, 20); UiAction::None }
            (KeyCode::PageDown, _) => { scroll_down(state, 20); UiAction::None }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                state.input.insert_char(c); update_suggestion(state); UiAction::None
            }
            _ => UiAction::None,
        }
    }

    fn normal_mode_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::BackTab, _) => UiAction::CyclePermissionMode,
            (KeyCode::Tab, _) => { state.input.mode = InputMode::Insert; UiAction::None }
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
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => { scroll_up(state, 3); UiAction::None }
            (KeyCode::Char('j'), _) | (KeyCode::Down, _) => { scroll_down(state, 3); UiAction::None }
            (KeyCode::Char('K'), _) => { state.input.history_prev(); UiAction::None }
            (KeyCode::Char('J'), _) => { state.input.history_next(); UiAction::None }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => { state.input.clear(); UiAction::None }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => { scroll_down(state, 20); UiAction::None }
            (KeyCode::PageUp, _) => { scroll_up(state, 20); UiAction::None }
            (KeyCode::PageDown, _) => { scroll_down(state, 20); UiAction::None }
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
