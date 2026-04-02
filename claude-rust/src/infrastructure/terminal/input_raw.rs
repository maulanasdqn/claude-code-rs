use std::sync::{Arc, Mutex, OnceLock, atomic::AtomicU8};

use crossterm::event;

use super::input_draw::redraw_input;
use super::input_handler::process_key;
use super::input_search::history_search;
use super::input_suggest::{clear_suggestions, get_suggestions, render_suggestions};

// ── Pasted image store ───────────────────────────────────────────────
/// Each entry is (media_type, base64_data).
static PASTED_IMAGES: OnceLock<Mutex<Vec<(String, String)>>> = OnceLock::new();

fn image_store() -> &'static Mutex<Vec<(String, String)>> {
    PASTED_IMAGES.get_or_init(|| Mutex::new(Vec::new()))
}

/// Store a pasted image and return its 1-based index.
fn store_image(media_type: String, data: String) -> usize {
    let mut store = image_store().lock().unwrap();
    store.push((media_type, data));
    store.len()
}

/// Take all pasted images, clearing the store.
pub fn take_pasted_images() -> Vec<(String, String)> {
    std::mem::take(&mut *image_store().lock().unwrap())
}

/// Clear the pasted image store without returning.
pub fn clear_pasted_images() {
    image_store().lock().unwrap().clear();
}

/// Returns `(result, extra_lines)` where `extra_lines` is the number of
/// newlines in the buffer at the time of submission (0 for single-line input).
/// Caller uses this to know how many content lines were rendered so it can
/// precisely clear the 4-line input box.
pub(super) fn read_line_raw(
    inner_width: usize,
    mode: &Arc<AtomicU8>,
    history: &[String],
    skill_names: &[String],
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
    // Track whether Esc was pressed to suppress re-showing suggestions
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
            ev,
            &mut buf,
            &mut cursor_pos,
            &mut hist_idx,
            &mut saved_buf,
            history,
            inner_width,
            mode,
            &mut extra_lines,
            &mut vim_mode,
            &suggestions,
            &mut suggestion_idx,
            &mut stash,
        ) {
            // Clear suggestion display before returning
            if prev_suggestion_count > 0 {
                clear_suggestions(prev_suggestion_count, inner_width);
            }

            // Handle special internal signals (prefixed with \x00)
            if let Some(ref text) = result {
                if text == "\x00history_search" {
                    // Clear suggestions before entering search mode
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
                    // Clear suggestions before attempting paste
                    if prev_suggestion_count > 0 {
                        clear_suggestions(prev_suggestion_count, inner_width);
                        prev_suggestion_count = 0;
                    }
                    suggestions.clear();

                    match read_clipboard_image() {
                        Some((media_type, data)) => {
                            let idx = store_image(media_type, data);
                            let marker = format!("[Image #{idx}]");
                            // Insert marker at cursor position
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

        // Detect if Esc was pressed to clear suggestions
        if old_suggestion_idx.is_some() && suggestion_idx.is_none() && !suggestions.is_empty() {
            // Esc or similar cleared the selection — hide suggestions
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

        // Recompute suggestions if buf changed
        if buf != prev_buf {
            esc_cleared = false; // buf changed, allow suggestions again
            prev_buf = buf.clone();

            let new_suggestions = get_suggestions(&buf, skill_names);
            let had_prev = prev_suggestion_count > 0;

            // Clear old suggestion display if count changed or suggestions disappeared
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
                // Suggestions disappeared — fix cursor after clear
                redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
            }
        } else if !suggestions.is_empty() && !esc_cleared {
            // Buf didn't change but suggestion_idx may have (arrow keys)
            // Re-render to update highlight
            if prev_suggestion_count > 0 {
                clear_suggestions(prev_suggestion_count, inner_width);
            }
            render_suggestions(&suggestions, suggestion_idx, inner_width);
            prev_suggestion_count = suggestions.len();
            redraw_input(&buf, cursor_pos, extra_lines, inner_width, mode);
        }
    }
}

/// Try to read an image from the system clipboard.
/// Returns `Some((media_type, base64_data))` if an image is found.
fn read_clipboard_image() -> Option<(String, String)> {
    use base64::Engine;

    let tmp = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
    let tmp_path = format!("{}/claude-rust-paste-{}.png", tmp.trim_end_matches('/'), std::process::id());

    // Wayland
    if let Ok(out) = std::process::Command::new("wl-paste")
        .args(["--type", "image/png", "--no-newline"])
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success() && !out.stdout.is_empty() {
            let enc = base64::engine::general_purpose::STANDARD.encode(&out.stdout);
            return Some(("image/png".to_string(), enc));
        }

    // X11
    if let Ok(out) = std::process::Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "image/png", "-o"])
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success() && !out.stdout.is_empty() {
            let enc = base64::engine::general_purpose::STANDARD.encode(&out.stdout);
            return Some(("image/png".to_string(), enc));
        }

    // macOS — write clipboard PNG to a temp file via osascript
    let script = format!(
        "try\n\
         set imgData to (the clipboard as «class PNGf»)\n\
         set fileRef to open for access POSIX file \"{tmp_path}\" with write permission\n\
         set eof fileRef to 0\n\
         write imgData to fileRef\n\
         close access fileRef\n\
         return \"ok\"\n\
         on error\n\
         return \"err\"\n\
         end try"
    );
    if let Ok(out) = std::process::Command::new("osascript")
        .arg("-e").arg(&script)
        .stderr(std::process::Stdio::null())
        .output()
        && String::from_utf8_lossy(&out.stdout).trim() == "ok"
            && let Ok(data) = std::fs::read(&tmp_path) {
                let _ = std::fs::remove_file(&tmp_path);
                if !data.is_empty() {
                    let enc = base64::engine::general_purpose::STANDARD.encode(&data);
                    return Some(("image/png".to_string(), enc));
                }
            }

    // macOS — pngpaste fallback (brew install pngpaste)
    if let Ok(out) = std::process::Command::new("pngpaste")
        .arg(&tmp_path)
        .stderr(std::process::Stdio::null())
        .output()
        && out.status.success()
            && let Ok(data) = std::fs::read(&tmp_path) {
                let _ = std::fs::remove_file(&tmp_path);
                if !data.is_empty() {
                    let enc = base64::engine::general_purpose::STANDARD.encode(&data);
                    return Some(("image/png".to_string(), enc));
                }
            }

    None
}
