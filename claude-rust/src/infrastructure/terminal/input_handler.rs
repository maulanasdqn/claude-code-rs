use std::sync::{Arc, atomic::AtomicU8};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use claude_rust_types::PermissionMode;

use super::input_border::redraw_status_bar;
use super::input_draw::{redraw_input, redraw_input_line};
use super::input_vim::process_vim_char;

/// Returns `Some(Some(text))` to submit, `Some(None)` for EOF/quit, `None` to keep reading.
///
/// When `suggestions` is non-empty, Up/Down navigate suggestions instead of history,
/// Tab/Enter accept the selected suggestion, and Esc clears suggestions.
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
        // ── Alt+Enter: insert newline ──────────────────────────────
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

        // ── Enter: accept suggestion or submit ─────────────────────
        Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
            if has_suggestions {
                if let Some(idx) = *suggestion_idx {
                    // Accept selected suggestion and submit
                    *buf = suggestions[idx].0.clone();
                    if !buf.ends_with(' ') {
                        buf.push(' ');
                    }
                    *cursor_pos = buf.len();
                    *suggestion_idx = None;
                    return Some(Some(buf.trim().to_string()));
                }
            }
            return Some(Some(buf.trim().to_string()));
        }

        // ── Esc: clear suggestions or toggle vim ───────────────────
        Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
            if has_suggestions {
                // Clear suggestions — signal handled by returning None
                // The caller (input_raw) will detect suggestion_idx = None and clear
                *suggestion_idx = None;
                // Force buf change to trigger suggestion recompute (clear)
                // We append and remove a nul to force "changed" detection
                // Actually, just return a special signal - we use a marker
                // The simplest approach: set buf to itself (no-op) but the caller
                // will see suggestions should be cleared because we return None
                // and the buf no longer warrants suggestions after Esc
                // Let's just mark that we want to clear by setting a flag via buf
                // Actually the cleanest way: just redraw and the event loop
                // will recompute suggestions (which will still match).
                // So instead, we need the event loop to track "force_clear"
                // Let's use suggestion_idx = None as the signal: if previously
                // Some and now None, the event loop clears.
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            } else {
                *vim_mode = !*vim_mode;
                redraw_input_line(buf, *cursor_pos, inner_width, mode);
            }
        }

        // ── Ctrl+D: EOF ────────────────────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if buf.is_empty() {
                return Some(None);
            }
        }

        // ── Ctrl+C: cancel ─────────────────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if buf.is_empty() {
                // Empty buffer — exit (same as Ctrl+D)
                return Some(None);
            }
            // Has text — clear the line (standard terminal behavior)
            buf.clear();
            *cursor_pos = 0;
            *extra_lines = 0;
            *suggestion_idx = None;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── Ctrl+U: clear line ─────────────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            buf.clear();
            *cursor_pos = 0;
            *extra_lines = 0;
            *suggestion_idx = None;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── Ctrl+W: delete word ────────────────────────────────────
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

        // ── Backspace ──────────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                buf.remove(*cursor_pos);
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Delete ─────────────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Delete, .. }) => {
            if *cursor_pos < buf.len() {
                buf.remove(*cursor_pos);
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Left ───────────────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Left, .. }) => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Right ──────────────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
            if *cursor_pos < buf.len() {
                *cursor_pos += 1;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Home / Ctrl+A ──────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Home, .. })
        | Event::Key(KeyEvent { code: KeyCode::Char('a'), modifiers: KeyModifiers::CONTROL, .. }) => {
            *cursor_pos = 0;
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── End / Ctrl+E ───────────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::End, .. })
        | Event::Key(KeyEvent { code: KeyCode::Char('e'), modifiers: KeyModifiers::CONTROL, .. }) => {
            *cursor_pos = buf.len();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── Up: suggestion nav or history ──────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
            if has_suggestions {
                let len = suggestions.len();
                *suggestion_idx = Some(match *suggestion_idx {
                    None => len - 1,
                    Some(0) => len - 1,
                    Some(i) => i - 1,
                });
            } else if !history.is_empty() {
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

        // ── Down: suggestion nav or history ────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
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

        // ── Tab: accept suggestion or old completion ───────────────
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

        // ── Shift+Tab: cycle permission mode ───────────────────────
        Event::Key(KeyEvent { code: KeyCode::BackTab, .. }) => {
            let current = PermissionMode::load(mode);
            current.next().store(mode);
            redraw_status_bar(mode);
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── Ctrl+S: stash / restore buffer ──────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if buf.is_empty() {
                // Restore from stash
                if let Some(saved) = stash.take() {
                    *buf = saved;
                    *cursor_pos = buf.len();
                    *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                    redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
                }
            } else {
                // Stash current buffer
                *stash = Some(buf.clone());
                buf.clear();
                *cursor_pos = 0;
                *extra_lines = 0;
                *suggestion_idx = None;
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Ctrl+B: send as background task ────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if !buf.is_empty() {
                let text = buf.trim().to_string();
                buf.clear();
                *cursor_pos = 0;
                *extra_lines = 0;
                *suggestion_idx = None;
                return Some(Some(format!("\x00background:{text}")));
            }
        }

        // ── Ctrl+O: toggle transcript view ───────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            return Some(Some("\x00transcript".to_string()));
        }

        // ── Ctrl+T: toggle todos view ────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            return Some(Some("\x00todos".to_string()));
        }

        // ── Ctrl+R: history search ───────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            return Some(Some("\x00history_search".to_string()));
        }

        // ── Ctrl+L: clear screen ─────────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            // Clear screen, cursor to top-left
            print!("\x1b[2J\x1b[H");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
        }

        // ── Ctrl+G: open external editor ─────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            if let Some(text) = open_external_editor(buf) {
                *buf = text;
                *cursor_pos = buf.len();
                *extra_lines = buf.chars().filter(|&c| c == '\n').count();
                redraw_input(buf, *cursor_pos, *extra_lines, inner_width, mode);
            }
        }

        // ── Ctrl+V: paste image from clipboard ───────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) => {
            return Some(Some("\x00paste_image".to_string()));
        }

        // ── Alt+P: open model picker ─────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: KeyModifiers::ALT,
            ..
        }) => {
            return Some(Some("/model".to_string()));
        }

        // ── Alt+O: toggle fast mode ──────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::ALT,
            ..
        }) => {
            return Some(Some("/fast".to_string()));
        }

        // ── Alt+T: toggle thinking ──────────────────────────────
        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::ALT,
            ..
        }) => {
            return Some(Some("/think".to_string()));
        }

        // ── Regular character ──────────────────────────────────────
        Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers, .. })
            if !modifiers.contains(KeyModifiers::CONTROL)
                && !modifiers.contains(KeyModifiers::ALT) =>
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

/// Open $VISUAL / $EDITOR with the current buffer, return edited text.
fn open_external_editor(current: &str) -> Option<String> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());

    let tmp = std::env::temp_dir().join(format!("claude-rust-edit-{}.txt", std::process::id()));
    std::fs::write(&tmp, current).ok()?;

    // Leave raw mode so the editor gets normal terminal
    crossterm::terminal::disable_raw_mode().ok();

    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (bin, args) = parts.split_first()?;
    let status = std::process::Command::new(bin)
        .args(args)
        .arg(&tmp)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .ok()?;

    // Re-enter raw mode
    crossterm::terminal::enable_raw_mode().ok();

    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }

    let text = std::fs::read_to_string(&tmp).ok()?;
    let _ = std::fs::remove_file(&tmp);
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}
