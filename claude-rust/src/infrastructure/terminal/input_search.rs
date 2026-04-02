use std::io::{self, Write};
use std::sync::{Arc, atomic::AtomicU8};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use super::{BOLD, CYAN, DIM, RESET, YELLOW};

/// Interactive reverse history search (Ctrl+R).
/// Returns `Some(matched_entry)` on accept, `None` on cancel.
pub(super) fn history_search(
    history: &[String],
    inner_width: usize,
    _mode: &Arc<AtomicU8>,
) -> Option<String> {
    if history.is_empty() {
        return None;
    }

    let mut query = String::new();
    let mut match_idx: usize = 0; // which match we're on (0 = most recent)

    draw_search(&query, history, match_idx, inner_width);

    loop {
        if !event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => break,
        };

        match ev {
            // Ctrl+R: cycle to next (older) match
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                match_idx += 1;
                draw_search(&query, history, match_idx, inner_width);
            }

            // Enter: accept match and submit
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                clear_search(inner_width);
                let matched = find_match(history, &query, match_idx);
                return matched;
            }

            // Esc or Tab: accept match into buffer (don't submit)
            Event::Key(KeyEvent { code: KeyCode::Esc, .. })
            | Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
                clear_search(inner_width);
                let matched = find_match(history, &query, match_idx);
                return matched.or(None);
            }

            // Ctrl+C: cancel search
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                clear_search(inner_width);
                return None;
            }

            // Backspace: delete last char from query
            Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                query.pop();
                match_idx = 0;
                draw_search(&query, history, match_idx, inner_width);
            }

            // Regular character: add to query
            Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers, .. })
                if !modifiers.contains(KeyModifiers::CONTROL)
                    && !modifiers.contains(KeyModifiers::ALT) =>
            {
                query.push(c);
                match_idx = 0;
                draw_search(&query, history, match_idx, inner_width);
            }

            _ => {}
        }
    }

    clear_search(inner_width);
    None
}

fn find_match(history: &[String], query: &str, skip: usize) -> Option<String> {
    if query.is_empty() {
        return history.last().map(|s| s.clone());
    }
    let lower_query = query.to_lowercase();
    history
        .iter()
        .rev()
        .filter(|entry| entry.to_lowercase().contains(&lower_query))
        .nth(skip)
        .cloned()
}

fn draw_search(query: &str, history: &[String], match_idx: usize, inner_width: usize) {
    let matched = find_match(history, query, match_idx);
    let avail = inner_width.saturating_sub(4);

    // Reserve 2 lines below current position (search prompt + match display)
    print!("\n\n");
    print!("\x1b[2A");

    // Line 1: search prompt
    print!("\x1b[B\r\x1b[2K");
    let prompt_text = format!("(reverse-i-search)`{query}': ");
    let truncated_prompt = if prompt_text.len() > avail {
        format!("{}...", &prompt_text[..avail.saturating_sub(3)])
    } else {
        prompt_text
    };
    print!("  {YELLOW}{BOLD}{truncated_prompt}{RESET}");

    // Line 2: matched entry
    print!("\x1b[B\r\x1b[2K");
    if let Some(ref m) = matched {
        let display = if m.len() > avail {
            format!("{}...", &m[..avail.saturating_sub(3)])
        } else {
            m.clone()
        };
        // Highlight matching portion
        if !query.is_empty() {
            let lower = display.to_lowercase();
            let lower_q = query.to_lowercase();
            if let Some(pos) = lower.find(&lower_q) {
                let before = &display[..pos];
                let matched_text = &display[pos..pos + query.len()];
                let after = &display[pos + query.len()..];
                print!("  {DIM}{before}{RESET}{CYAN}{BOLD}{matched_text}{RESET}{DIM}{after}{RESET}");
            } else {
                print!("  {DIM}{display}{RESET}");
            }
        } else {
            print!("  {DIM}{display}{RESET}");
        }
    } else if !query.is_empty() {
        print!("  {DIM}(no match){RESET}");
    }

    // Move back up to content line
    print!("\x1b[2A");

    io::stdout().flush().ok();
}

fn clear_search(inner_width: usize) {
    // Clear the 2 search lines below
    print!("\x1b[B\r\x1b[2K");
    print!("\x1b[B\r\x1b[2K");

    // Redraw the bottom border on line 1 (right below content)
    print!("\x1b[2A");
    print!("\x1b[B\r\x1b[2K  {DIM}\u{2570}{}{}\u{256F}{RESET}", "\u{2500}".repeat(inner_width), "");
    print!("\x1b[1A");

    io::stdout().flush().ok();
}
