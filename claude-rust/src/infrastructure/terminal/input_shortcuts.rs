use std::sync::{Arc, atomic::AtomicU8};

use super::input_border::redraw_status_bar;
use super::input_draw::redraw_input;

pub(super) fn nav_up(
    has_suggestions: bool,
    suggestions: &[(String, String)],
    suggestion_idx: &mut Option<usize>,
    history: &[String],
    hist_idx: &mut Option<usize>,
    saved_buf: &mut String,
    buf: &mut String,
    cursor_pos: &mut usize,
    extra_lines: &mut usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    if has_suggestions {
        let len = suggestions.len();
        *suggestion_idx = Some(match *suggestion_idx {
            None => len - 1,
            Some(0) => len - 1,
            Some(i) => i - 1,
        });
    } else if !history.is_empty() {
        let next_idx = match *hist_idx {
            None => { *saved_buf = buf.clone(); history.len() - 1 }
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

pub(super) fn nav_down(
    has_suggestions: bool,
    suggestions: &[(String, String)],
    suggestion_idx: &mut Option<usize>,
    history: &[String],
    hist_idx: &mut Option<usize>,
    saved_buf: &mut String,
    buf: &mut String,
    cursor_pos: &mut usize,
    extra_lines: &mut usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    if has_suggestions {
        let len = suggestions.len();
        *suggestion_idx = Some(match *suggestion_idx {
            None => 0,
            Some(i) if i + 1 >= len => 0,
            Some(i) => i + 1,
        });
    } else {
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
}

pub(super) fn handle_ctrl_shortcut(
    c: char,
    buf: &mut String,
    cursor_pos: &mut usize,
    extra_lines: &mut usize,
    suggestion_idx: &mut Option<usize>,
    stash: &mut Option<String>,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) -> Option<Option<String>> {
    match c {
        'd' => { if buf.is_empty() { return Some(None); } }
        'c' => {
            if buf.is_empty() { return Some(None); }
            buf.clear(); *cursor_pos = 0; *extra_lines = 0; *suggestion_idx = None;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        'u' => {
            buf.clear(); *cursor_pos = 0; *extra_lines = 0; *suggestion_idx = None;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        'w' => {
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
        's' => {
            if buf.is_empty() {
                if let Some(saved) = stash.take() {
                    *buf = saved; *cursor_pos = buf.len();
                    *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                    redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
                }
            } else {
                *stash = Some(buf.clone());
                buf.clear(); *cursor_pos = 0; *extra_lines = 0; *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        'b' => {
            if !buf.is_empty() {
                let text = buf.trim().to_string();
                buf.clear(); *cursor_pos = 0; *extra_lines = 0; *suggestion_idx = None;
                return Some(Some(format!("\x00background:{text}")));
            }
        }
        'a' => { *cursor_pos = 0; redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode); }
        'e' => { *cursor_pos = buf.len(); redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode); }
        'o' => return Some(Some("\x00transcript".to_string())),
        't' => return Some(Some("\x00todos".to_string())),
        'r' => return Some(Some("\x00history_search".to_string())),
        'l' => {
            print!("\x1b[2J\x1b[H");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }
        'g' => {
            if let Some(text) = open_external_editor(buf) {
                *buf = text; *cursor_pos = buf.len();
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }
        'v' => return Some(Some("\x00paste_image".to_string())),
        _ => {}
    }
    None
}

pub(super) fn handle_shift_tab(buf: &str, cursor_pos: usize, extra_lines: usize, inner_width: usize, mode: &Arc<AtomicU8>) {
    use claude_rust_types::PermissionMode;
    let current = PermissionMode::load(mode);
    current.next().store(mode);
    redraw_status_bar(mode);
    redraw_input(buf, cursor_pos, extra_lines, inner_width, mode);
}

fn open_external_editor(current: &str) -> Option<String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());
    let tmp = std::env::temp_dir().join(format!("claude-rust-edit-{}.txt", std::process::id()));
    std::fs::write(&tmp, current).ok()?;
    crossterm::terminal::disable_raw_mode().ok();
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (bin, args) = parts.split_first()?;
    let status = std::process::Command::new(bin)
        .args(args).arg(&tmp)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status().ok()?;
    crossterm::terminal::enable_raw_mode().ok();
    if !status.success() { let _ = std::fs::remove_file(&tmp); return None; }
    let text = std::fs::read_to_string(&tmp).ok()?;
    let _ = std::fs::remove_file(&tmp);
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}
