use std::sync::{Arc, atomic::AtomicU8};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::input_draw::{redraw_input, redraw_input_line};
use super::input_shortcuts::{handle_ctrl_shortcut, handle_shift_tab, nav_down, nav_up};
use super::input_vim::process_vim_char;

#[allow(clippy::too_many_arguments)]
pub(super) fn process_key(
    ev: Event,
    buf: &mut String,
    cursor_pos: &mut usize,
    hist_idx: &mut Option<usize>,
    saved_buf: &mut String,
    history: &[String],
    inner_width: usize,
    mode: &Arc<AtomicU8>,
    extra_lines: &mut usize,
    vim_mode: &mut bool,
    suggestions: &[(String, String)],
    suggestion_idx: &mut Option<usize>,
    stash: &mut Option<String>,
) -> Option<Option<String>> {
    let has_suggestions = !suggestions.is_empty();

    match ev {
        Event::Key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::ALT, .. }) => {
            buf.insert(*cursor_pos, '\n');
            *cursor_pos += 1;
            *extra_lines = buf.chars().filter(|&c| c == '\n').count();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
            if has_suggestions && let Some(idx) = *suggestion_idx {
                *buf = suggestions[idx].0.clone();
                if !buf.ends_with(' ') { buf.push(' '); }
                *cursor_pos = buf.len();
                *suggestion_idx = None;
                return Some(Some(buf.trim().to_string()));
            }
            return Some(Some(buf.trim().to_string()));
        }

        Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
            if has_suggestions {
                *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            } else {
                *vim_mode = !*vim_mode;
                redraw_input_line(buf, *cursor_pos, inner_width, mode);
            }
        }

        Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL, .. }) => {
            return handle_ctrl_shortcut(c, buf, cursor_pos, extra_lines, suggestion_idx, stash, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                buf.remove(*cursor_pos);
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                *suggestion_idx = None;
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
            if *cursor_pos > 0 { *cursor_pos -= 1; redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode); }
        }

        Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
            if *cursor_pos < buf.len() { *cursor_pos += 1; redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode); }
        }

        Event::Key(KeyEvent { code: KeyCode::Home, .. }) => {
            *cursor_pos = 0;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::End, .. }) => {
            *cursor_pos = buf.len();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
            nav_up(has_suggestions, suggestions, suggestion_idx, history, hist_idx, saved_buf, buf, cursor_pos, extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
            nav_down(has_suggestions, suggestions, suggestion_idx, history, hist_idx, saved_buf, buf, cursor_pos, extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
            if has_suggestions {
                let idx = suggestion_idx.unwrap_or(0);
                *buf = suggestions[idx].0.clone();
                buf.push(' ');
                *cursor_pos = buf.len();
                *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        Event::Key(KeyEvent { code: KeyCode::BackTab, .. }) => {
            handle_shift_tab(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        Event::Key(KeyEvent { code: KeyCode::Char('p'), modifiers: KeyModifiers::ALT, .. }) => {
            return Some(Some("/model".to_string()));
        }

        Event::Key(KeyEvent { code: KeyCode::Char('o'), modifiers: KeyModifiers::ALT, .. }) => {
            return Some(Some("/fast".to_string()));
        }

        Event::Key(KeyEvent { code: KeyCode::Char('t'), modifiers: KeyModifiers::ALT, .. }) => {
            return Some(Some("/think".to_string()));
        }

        Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers, .. })
            if !modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::ALT) =>
        {
            if *vim_mode {
                process_vim_char(c, buf, cursor_pos, vim_mode, inner_width, mode);
            } else {
                buf.insert(*cursor_pos, c);
                *cursor_pos += 1;
                *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        _ => {}
    }
    None
}
