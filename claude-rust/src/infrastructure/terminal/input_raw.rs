use std::sync::{Arc, Mutex, OnceLock, atomic::AtomicU8};

use crossterm::event;

use super::input_clipboard::read_clipboard_image;
use super::input_draw::redraw_input;
use super::input_handler::process_key;
use super::input_search::history_search;
use super::input_suggest::{clear_suggestions, get_suggestions, render_suggestions};

static PASTED_IMAGES: OnceLock<Mutex<Vec<(String, String)>>> = OnceLock::new();

fn image_store() -> &'static Mutex<Vec<(String, String)>> {
    PASTED_IMAGES.get_or_init(|| Mutex::new(Vec::new()))
}

fn store_image(media_type: String, data: String) -> usize {
    let mut store = image_store().lock().unwrap();
    store.push((media_type, data));
    store.len()
}

pub fn take_pasted_images() -> Vec<(String, String)> {
    std::mem::take(&mut *image_store().lock().unwrap())
}

pub fn clear_pasted_images() {
    image_store().lock().unwrap().clear();
}

pub(super) fn read_line_raw(
    inner_width: usize,
    mode: &Arc<AtomicU8>,
    history: &[String],
    skill_names: &[(String, String)],
) -> (Option<String>, usize) {
    let mut buf = String::new();
    let mut cursor_pos: usize = 0;
    let mut hist_idx: Option<usize> = None;
    let mut saved_buf = String::new();
    let mut extra_lines: usize = 0;
    let mut vim_mode: bool = false;
    let mut stash: Option<String> = None;
    let mut suggestions: Vec<(String, String)> = Vec::new();
    let mut suggestion_idx: Option<usize> = None;
    let mut prev_suggestion_count: usize = 0;
    let mut prev_buf = String::new();
    let mut esc_cleared = false;

    loop {
        if !event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => return (None, extra_lines),
        };

        let old_suggestion_idx = suggestion_idx;

        if let Some(result) = process_key(
            ev, &mut buf, &mut cursor_pos, &mut hist_idx, &mut saved_buf,
            history, inner_width, mode, &mut extra_lines, &mut vim_mode,
            &suggestions, &mut suggestion_idx, &mut stash,
        ) {
            if prev_suggestion_count > 0 {
                clear_suggestions(prev_suggestion_count, inner_width);
            }
            if let Some(ref text) = result {
                if text == "\x00history_search" {
                    if prev_suggestion_count > 0 {
                        clear_suggestions(prev_suggestion_count, inner_width);
                        prev_suggestion_count = 0;
                    }
                    suggestions.clear();
                    if let Some(found) = history_search(history, inner_width, mode) {
                        buf = found;
                        cursor_pos = buf.len();
                        extra_lines = buf.chars().filter(|&c| c == '\n').count();
                    }
                    prev_buf = buf.clone();
                    redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
                    continue;
                }
                if text == "\x00paste_image" {
                    if prev_suggestion_count > 0 {
                        clear_suggestions(prev_suggestion_count, inner_width);
                        prev_suggestion_count = 0;
                    }
                    suggestions.clear();
                    match read_clipboard_image() {
                        Some((media_type, data)) => {
                            let idx = store_image(media_type, data);
                            let marker = format!("[Image #{idx}]");
                            buf.insert_str(cursor_pos, &marker);
                            cursor_pos += marker.len();
                            prev_buf = buf.clone();
                            redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
                            continue;
                        }
                        None => {
                            use std::io::Write;
                            print!("\r\x1b[K  \x1b[2mNo image in clipboard\x1b[0m");
                            let _ = std::io::stdout().flush();
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                            prev_buf = buf.clone();
                            redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
                            continue;
                        }
                    }
                }
            }
            return (result, extra_lines);
        }

        if old_suggestion_idx.is_some() && suggestion_idx.is_none() && !suggestions.is_empty() {
            if prev_suggestion_count > 0 {
                clear_suggestions(prev_suggestion_count, inner_width);
                prev_suggestion_count = 0;
            }
            suggestions.clear();
            esc_cleared = true;
            prev_buf = buf.clone();
            redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
            continue;
        }

        if buf != prev_buf {
            esc_cleared = false;
            prev_buf = buf.clone();
            let new_suggestions = get_suggestions(&buf, skill_names);
            let had_prev = prev_suggestion_count > 0;
            if had_prev && (new_suggestions.is_empty() || new_suggestions.len() != suggestions.len()) {
                clear_suggestions(prev_suggestion_count, inner_width);
                prev_suggestion_count = 0;
            }
            suggestions = new_suggestions;
            suggestion_idx = None;
            if !suggestions.is_empty() {
                render_suggestions(&suggestions, suggestion_idx, inner_width);
                prev_suggestion_count = suggestions.len();
                redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
            } else if had_prev {
                redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
            }
        } else if !suggestions.is_empty() && !esc_cleared {
            if prev_suggestion_count > 0 {
                clear_suggestions(prev_suggestion_count, inner_width);
            }
            render_suggestions(&suggestions, suggestion_idx, inner_width);
            prev_suggestion_count = suggestions.len();
            redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
        }
    }
}
