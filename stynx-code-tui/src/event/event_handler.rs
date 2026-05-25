use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};

use crate::dialogs::{open_command_palette, open_file_mention, open_model_picker, open_session_list};
use crate::state::{
    AppState, InputMode, ModalKind, PermissionChoice, SelectKind, filter_options,
};

#[derive(Debug, Clone)]
pub enum UiAction {
    Submit(String),
    CyclePermissionMode,
    Quit,
    SelectModel(String),
    SelectSession(String),
    PermissionDecision(PermissionChoice),
    RunCommand(String),
    ToggleSidebar,
    InputConfirmed { kind: crate::state::InputKind, value: String },
    Interrupt,
    None,
}

const COMMANDS: &[(&str, &str)] = &[
    ("/add",         "Pin a file path to every message"),
    ("/commit",      "Generate a commit message from the diff"),
    ("/compact",     "Compact context to save tokens"),
    ("/config",      "Show merged config"),
    ("/copy",        "Copy last response to clipboard"),
    ("/diff",        "Show git diff"),
    ("/effort",      "Set effort: low/medium/high/max"),
    ("/exit",        "Exit session"),
    ("/export",      "Export conversation to markdown"),
    ("/fast",        "Toggle fast mode (haiku)"),
    ("/files",       "List pinned files"),
    ("/help",        "Show help"),
    ("/intern",      "Delegate task to intern (DeepSeek)"),
    ("/memory",      "Show CLAUDE.md"),
    ("/mode",        "Cycle permission mode"),
    ("/model",       "Switch model"),
    ("/permissions", "Show allow / deny rules"),
    ("/plan",        "Toggle plan mode"),
    ("/quit",        "Exit session"),
    ("/review",      "Review current git diff"),
    ("/rewind",      "Remove last n exchanges"),
    ("/skills",      "List available skills"),
    ("/status",      "Show git status"),
    ("/think",       "Toggle extended thinking"),
    ("/usage",       "Show plan usage limits"),
    ("/version",     "Show version"),
];

fn update_suggestion(state: &mut AppState) {
    let buf = state.input.buffer.clone();
    if buf.starts_with('/') && !buf.contains(' ') {
        let matches: Vec<(String, String)> = COMMANDS
            .iter()
            .filter(|(c, _)| c.starts_with(buf.as_str()) && *c != buf.as_str())
            .map(|(c, d)| ((*c).to_string(), (*d).to_string()))
            .collect();
        state.input.suggestion = if matches.len() == 1 {
            matches[0].0[buf.len()..].to_string()
        } else {
            String::new()
        };
        state.input.slash_matches = matches;
        if state.input.slash_selected >= state.input.slash_matches.len() {
            state.input.slash_selected = 0;
        }
    } else {
        state.input.suggestion = String::new();
        state.input.slash_matches.clear();
        state.input.slash_selected = 0;
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

fn dispatch_palette(state: &mut AppState, command: &str) -> UiAction {
    match command {
        "session.list" => { open_session_list(state); UiAction::None }
        "session.new" => UiAction::RunCommand("session.new".into()),
        "session.compact" => UiAction::RunCommand("session.compact".into()),
        "session.export" => UiAction::RunCommand("session.export".into()),
        "session.rename" => UiAction::RunCommand("session.rename".into()),
        "status.show" => UiAction::RunCommand("status.show".into()),
        "skills.show" => UiAction::RunCommand("skills.show".into()),
        "model.list" => { open_model_picker(state); UiAction::None }
        "model.cycle_recent" => UiAction::RunCommand("model.cycle_recent".into()),
        "mode.cycle" => UiAction::CyclePermissionMode,
        "sidebar.toggle" => UiAction::ToggleSidebar,
        "theme.switch" => { crate::dialogs::open_theme_picker(state); UiAction::None }
        "help.show" => UiAction::RunCommand("help.show".into()),
        "app.quit" => UiAction::Quit,
        _ => UiAction::None,
    }
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }

    pub fn handle(event: Event, state: &mut AppState) -> UiAction {
        match event {
            Event::Mouse(m) => {
                match m.kind {
                    MouseEventKind::ScrollUp => scroll_up(state, 3),
                    MouseEventKind::ScrollDown => scroll_down(state, 3),
                    _ => {}
                }
                return UiAction::None;
            }
            Event::Paste(text) => {
                if text.len() > 200 || text.matches('\n').count() > 4 {
                    let lines = text.lines().count().max(1);
                    let chars = text.chars().count();
                    let token = format!("[Pasted {lines} lines, {chars} chars]");
                    for c in token.chars() {
                        state.input.insert_char(c);
                    }
                    state.input.pasted_buffer = Some(text);
                } else {
                    for c in text.chars() {
                        state.input.insert_char(c);
                    }
                }
                return UiAction::None;
            }
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press { return UiAction::None; }
                if state.modal.active.is_some() { return Self::modal_key(key, state); }
                if state.input.mode == InputMode::Normal {
                    return Self::normal_mode_key(key, state);
                }
                Self::insert_mode_key(key, state)
            }
            _ => UiAction::None,
        }
    }

    fn insert_mode_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                open_command_palette(state);
                UiAction::None
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                open_session_list(state);
                UiAction::None
            }
            (KeyCode::Char('m'), KeyModifiers::CONTROL) => {
                open_model_picker(state);
                UiAction::None
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => UiAction::ToggleSidebar,
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                state.tool_details = !state.tool_details;
                let label = if state.tool_details { "tool details on" } else { "tool details off" };
                state.toasts.info(label);
                UiAction::None
            }
            (KeyCode::Esc, _) => {
                if state.is_streaming {
                    return UiAction::Interrupt;
                }
                if !state.input.slash_matches.is_empty() {
                    state.input.slash_matches.clear();
                    state.input.slash_selected = 0;
                    return UiAction::None;
                }
                state.input.mode = InputMode::Normal;
                UiAction::None
            }
            (KeyCode::BackTab, _) => UiAction::CyclePermissionMode,
            (KeyCode::Tab, _) => { state.input.complete_suggestion(); UiAction::None }
            (KeyCode::Enter, m) if m.contains(KeyModifiers::SHIFT)
                || m.contains(KeyModifiers::ALT)
                || m.contains(KeyModifiers::CONTROL) =>
            {
                state.input.insert_char('\n'); UiAction::None
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                state.input.insert_char('\n'); UiAction::None
            }
            (KeyCode::Enter, _) => {
                if !state.input.slash_matches.is_empty() {
                    let i = state.input.slash_selected.min(state.input.slash_matches.len() - 1);
                    let cmd = state.input.slash_matches[i].0.clone();
                    state.input.buffer = format!("{cmd} ");
                    state.input.cursor_pos = state.input.buffer.len();
                    state.input.slash_matches.clear();
                    state.input.slash_selected = 0;
                    state.input.suggestion.clear();
                    return UiAction::None;
                }
                let display = state.input.buffer.trim().to_string();
                if display.is_empty() { return UiAction::None; }
                let real = state.input.expand_for_submit().trim().to_string();
                state.input.push_history(display);
                state.input.clear();
                UiAction::Submit(real)
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
            (KeyCode::Up, KeyModifiers::SHIFT) => { scroll_up(state, 1); UiAction::None }
            (KeyCode::Down, KeyModifiers::SHIFT) => { scroll_down(state, 1); UiAction::None }
            (KeyCode::Char('u'), m) if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::ALT) => {
                scroll_up(state, 10); UiAction::None
            }
            (KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::ALT) => {
                scroll_down(state, 10); UiAction::None
            }
            (KeyCode::Up, _) => {
                if !state.input.slash_matches.is_empty() {
                    let len = state.input.slash_matches.len();
                    state.input.slash_selected = if state.input.slash_selected == 0 {
                        len - 1
                    } else {
                        state.input.slash_selected - 1
                    };
                } else if state.input.buffer.is_empty() {
                    scroll_up(state, 3);
                } else if state.input.line_count() > 1 && state.input.cursor_up_line() {
                    // moved within multi-line buffer
                } else {
                    state.input.history_prev();
                }
                UiAction::None
            }
            (KeyCode::Down, _) => {
                if !state.input.slash_matches.is_empty() {
                    let len = state.input.slash_matches.len();
                    state.input.slash_selected = (state.input.slash_selected + 1) % len;
                } else if state.input.buffer.is_empty() {
                    scroll_down(state, 3);
                } else if state.input.line_count() > 1 && state.input.cursor_down_line() {
                    // moved within multi-line buffer
                } else {
                    state.input.history_next();
                }
                UiAction::None
            }
            (KeyCode::PageUp, _) => { scroll_up(state, 20); UiAction::None }
            (KeyCode::PageDown, _) => { scroll_down(state, 20); UiAction::None }
            (KeyCode::Char('@'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                state.input.insert_char('@');
                update_suggestion(state);
                open_file_mention(state);
                UiAction::None
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                state.input.insert_char(c); update_suggestion(state); UiAction::None
            }
            _ => UiAction::None,
        }
    }

    fn normal_mode_key(key: KeyEvent, state: &mut AppState) -> UiAction {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => UiAction::Quit,
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                open_command_palette(state);
                UiAction::None
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                open_session_list(state);
                UiAction::None
            }
            (KeyCode::Char('m'), KeyModifiers::CONTROL) => {
                open_model_picker(state);
                UiAction::None
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => UiAction::ToggleSidebar,
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
        // Special: simple modals that close on any non-typing key.
        if let Some(ModalKind::Info { .. }) = &state.modal.active {
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Enter, _) | (KeyCode::Char('q'), _) => {
                    state.modal.close();
                }
                _ => {}
            }
            return UiAction::None;
        }
        if let Some(ModalKind::Input { buffer, kind, .. }) = state.modal.active.as_mut() {
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    state.modal.close();
                    return UiAction::None;
                }
                (KeyCode::Enter, _) => {
                    let value = buffer.trim().to_string();
                    let k = *kind;
                    state.modal.close();
                    if value.is_empty() { return UiAction::None; }
                    return UiAction::InputConfirmed { kind: k, value };
                }
                (KeyCode::Backspace, _) => { buffer.pop(); return UiAction::None; }
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => { buffer.clear(); return UiAction::None; }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    buffer.push(c);
                    return UiAction::None;
                }
                _ => return UiAction::None,
            }
        }
        let active = state.modal.active.as_mut();
        let Some(active) = active else { return UiAction::None; };
        match active {
            ModalKind::Info { .. } | ModalKind::Input { .. } => UiAction::None,
            ModalKind::Permission { choice, .. } => match (key.code, key.modifiers) {
                (KeyCode::Esc, _) => {
                    state.modal.close();
                    UiAction::PermissionDecision(PermissionChoice::Reject)
                }
                (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                    *choice = match *choice {
                        PermissionChoice::Once => PermissionChoice::Reject,
                        PermissionChoice::Always => PermissionChoice::Once,
                        PermissionChoice::Reject => PermissionChoice::Always,
                    };
                    UiAction::None
                }
                (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                    *choice = match *choice {
                        PermissionChoice::Once => PermissionChoice::Always,
                        PermissionChoice::Always => PermissionChoice::Reject,
                        PermissionChoice::Reject => PermissionChoice::Once,
                    };
                    UiAction::None
                }
                (KeyCode::Enter, _) => {
                    let decision = *choice;
                    state.modal.close();
                    UiAction::PermissionDecision(decision)
                }
                (KeyCode::Char('y'), _) | (KeyCode::Char('Y'), _) => {
                    state.modal.close();
                    UiAction::PermissionDecision(PermissionChoice::Once)
                }
                (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) => {
                    state.modal.close();
                    UiAction::PermissionDecision(PermissionChoice::Always)
                }
                (KeyCode::Char('n'), _) | (KeyCode::Char('N'), _) => {
                    state.modal.close();
                    UiAction::PermissionDecision(PermissionChoice::Reject)
                }
                _ => UiAction::None,
            },
            ModalKind::Select {
                query,
                options,
                selected,
                kind,
                ..
            } => match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    state.modal.close();
                    UiAction::None
                }
                (KeyCode::Up, _) => {
                    let len = filter_options(options, query).len();
                    if len > 0 {
                        *selected = if *selected == 0 { len - 1 } else { *selected - 1 };
                    }
                    UiAction::None
                }
                (KeyCode::Down, _) => {
                    let len = filter_options(options, query).len();
                    if len > 0 {
                        *selected = (*selected + 1) % len;
                    }
                    UiAction::None
                }
                (KeyCode::PageUp, _) => {
                    *selected = selected.saturating_sub(10);
                    UiAction::None
                }
                (KeyCode::PageDown, _) => {
                    let len = filter_options(options, query).len();
                    if len > 0 {
                        *selected = (*selected + 10).min(len - 1);
                    }
                    UiAction::None
                }
                (KeyCode::Backspace, _) => {
                    query.pop();
                    *selected = 0;
                    UiAction::None
                }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    query.push(c);
                    *selected = 0;
                    UiAction::None
                }
                (KeyCode::Enter, _) => {
                    let filtered = filter_options(options, query);
                    let Some(&orig) = filtered.get(*selected) else {
                        state.modal.close();
                        return UiAction::None;
                    };
                    let value = options[orig].value.clone();
                    let kind = *kind;
                    state.modal.close();
                    match kind {
                        SelectKind::CommandPalette => dispatch_palette(state, &value),
                        SelectKind::ModelPicker => UiAction::SelectModel(value),
                        SelectKind::SessionList => {
                            if value == "__empty__" {
                                UiAction::None
                            } else {
                                UiAction::SelectSession(value)
                            }
                        }
                        SelectKind::Help => UiAction::None,
                        SelectKind::SkillPicker => {
                            if let Some(name) = value.strip_prefix("skill:") {
                                state.input.buffer = format!("/{name} ");
                                state.input.cursor_pos = state.input.buffer.len();
                                state.input.mode = InputMode::Insert;
                            }
                            UiAction::None
                        }
                        SelectKind::FileMention => {
                            if value != "__empty__" {
                                // Buffer already contains "@" — append the path then a space
                                for c in value.chars() {
                                    state.input.insert_char(c);
                                }
                                state.input.insert_char(' ');
                            }
                            state.input.mode = InputMode::Insert;
                            UiAction::None
                        }
                        SelectKind::ThemePicker => {
                            if crate::theme::set_theme(&value) {
                                state.toasts.success(format!("theme → {value}"));
                            }
                            UiAction::None
                        }
                    }
                }
                _ => UiAction::None,
            },
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self { Self }
}
