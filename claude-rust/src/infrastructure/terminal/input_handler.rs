use std::sync::{Arc, atomic::AtomicU8};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use claude_rust_types::PermissionMode;

use super::input_border::redraw_top_border;
use super::input_draw::{redraw_input, redraw_input_line};
use super::input_vim::process_vim_char;

const BUILTIN_COMMANDS: &[&str] = &[
    "/help", "/clear", "/compact", "/cost", "/diff", "/status", "/doctor", "/config",
    "/permissions", "/session", "/plan", "/mode", "/model", "/think", "/init", "/add",
    "/files", "/skills", "/memory", "/export", "/review", "/commit", "/fast", "/rewind", "/quit",
];

pub(super) fn process_key(
    ev: Event,
    buf: &mut String,
    cursor_pos: &mut usize,
    hist_idx: &mut Option<usize>,
    saved_buf: &mut String,
    history: &[String],
    skill_names: &[String],
    inner_width: usize,
    mode: &Arc<AtomicU8>,
    extra_lines: &mut usize,
    vim_mode: &mut bool,
) -> Option<Option<String>> {
    match ev {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::ALT,
            ..
        }) => {
            buf.insert(*cursor_pos, '\n');
            *cursor_pos += 1;
            *extra_lines = buf.chars().filter(|&c| c == '\n').count();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
            return Some(Some(buf.trim().to_string()));
        }
        Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
            *vim_mode = !*vim_mode;
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if buf.is_empty() {
                return Some(None);
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            return Some(Some(String::new()));
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            buf.clear();
            *cursor_pos = 0;
            *extra_lines = 0;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if *cursor_pos > 0 {
                let before = &buf[..*cursor_pos];
                let trimmed = before.trim_end();
                let new_end = trimmed.rfind(|c: char| c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                buf.drain(new_end..*cursor_pos);
                *cursor_pos = new_end;
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                buf.remove(*cursor_pos);
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Delete, .. }) => {
            if *cursor_pos < buf.len() {
                buf.remove(*cursor_pos);
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Left, .. }) => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
            if *cursor_pos < buf.len() {
                *cursor_pos += 1;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Home, .. })
        | Event::Key(KeyEvent { code: KeyCode::Char('a'), modifiers: KeyModifiers::CONTROL, .. }) => {
            *cursor_pos = 0;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        Event::Key(KeyEvent { code: KeyCode::End, .. })
        | Event::Key(KeyEvent { code: KeyCode::Char('e'), modifiers: KeyModifiers::CONTROL, .. }) => {
            *cursor_pos = buf.len();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
            if !history.is_empty() {
                let next_idx = match *hist_idx {
                    None => {
                        *saved_buf = buf.clone();
                        history.len() - 1
                    }
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                *hist_idx = Some(next_idx);
                *buf = history[next_idx].clone();
                *cursor_pos = buf.len();
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
            match *hist_idx {
                None => {}
                Some(i) if i + 1 >= history.len() => {
                    *hist_idx = None;
                    *buf = saved_buf.clone();
                    *cursor_pos = buf.len();
                    *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                    redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
                }
                Some(i) => {
                    *hist_idx = Some(i + 1);
                    *buf = history[i + 1].clone();
                    *cursor_pos = buf.len();
                    *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                    redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
                }
            }
        }
        Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
            if buf.starts_with('/') {
                let skill_cmds = skill_names.iter().map(|s| format!("/{s}"));
                let matches: Vec<String> = BUILTIN_COMMANDS.iter().map(|s| s.to_string())
                    .chain(skill_cmds)
                    .filter(|cmd| cmd.starts_with(buf.as_str()))
                    .collect();
                if matches.len() == 1 {
                    *buf = matches[0].clone();
                    *cursor_pos = buf.len();
                    redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
                }
            }
        }
        Event::Key(KeyEvent { code: KeyCode::BackTab, .. }) => {
            let current = PermissionMode::load(mode);
            current.next().store(mode);
            redraw_top_border(inner_width, mode);
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers, .. })
            if !modifiers.contains(KeyModifiers::CONTROL)
                && !modifiers.contains(KeyModifiers::ALT) =>
        {
            if *vim_mode {
                process_vim_char(c, buf, cursor_pos, vim_mode, inner_width, mode);
            } else {
                buf.insert(*cursor_pos, c);
                *cursor_pos += 1;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        _ => {}
    }
    None
}
