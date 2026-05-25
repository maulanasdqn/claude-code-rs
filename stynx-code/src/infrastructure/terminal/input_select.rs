use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal;

use super::{BOLD, CYAN, DIM, GREEN, RESET};

/// Show an interactive selection list. Returns `Some(value)` on Enter, `None` on Esc.
///
/// `items` is a slice of `(value, label)` pairs.
/// `current` is the currently active value (shown with a green tag).
pub fn select_from_list(
    title: &str,
    items: &[(String, String)],
    current: &str,
) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let mut idx: usize = items
        .iter()
        .position(|(v, _)| v == current)
        .unwrap_or(0);

    terminal::enable_raw_mode().ok()?;

    // Print initial blank lines to reserve space, then move back up
    let total_lines = items.len() + 4;
    for _ in 0..total_lines {
        println!();
    }
    print!("\x1b[{}A", total_lines);
    io::stdout().flush().ok();

    draw_list(title, items, idx, current, total_lines);

    let result = loop {
        if !event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        match event::read() {
            Ok(Event::Key(KeyEvent { code: KeyCode::Up, .. })) => {
                idx = if idx == 0 { items.len() - 1 } else { idx - 1 };
                draw_list(title, items, idx, current, total_lines);
            }
            Ok(Event::Key(KeyEvent { code: KeyCode::Down, .. })) => {
                idx = (idx + 1) % items.len();
                draw_list(title, items, idx, current, total_lines);
            }
            Ok(Event::Key(KeyEvent { code: KeyCode::Enter, .. })) => {
                break Some(items[idx].0.clone());
            }
            Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. })) => {
                break None;
            }
            Ok(Event::Key(KeyEvent { code: KeyCode::Char(c), .. })) => {
                if let Some(digit) = c.to_digit(10) {
                    let n = digit as usize;
                    if n >= 1 && n <= items.len() {
                        break Some(items[n - 1].0.clone());
                    }
                }
            }
            _ => {}
        }
    };

    // Clear the selection UI (cursor is at top of reserved area from draw_list)
    print!("\r\x1b[2K");
    for _ in 1..total_lines {
        print!("\x1b[B\r\x1b[2K");
    }
    // Move back up to where we started
    print!("\x1b[{}A", total_lines - 1);
    io::stdout().flush().ok();

    terminal::disable_raw_mode().ok();

    result
}

fn draw_list(
    title: &str,
    items: &[(String, String)],
    selected: usize,
    current: &str,
    total_lines: usize,
) {
    // Draw from top of reserved area (cursor is already here).
    // Use \x1b[B (cursor down) instead of \n to avoid terminal scrolling.

    // Line 0: blank
    print!("\r\x1b[2K");

    // Line 1: title
    print!("\x1b[B\r\x1b[2K  {BOLD}{title}{RESET}");

    // Line 2: blank
    print!("\x1b[B\r\x1b[2K");

    for (i, (value, label)) in items.iter().enumerate() {
        print!("\x1b[B\r\x1b[2K");
        let is_current = value == current;
        let tag = if is_current {
            format!(" {GREEN}(current){RESET}")
        } else {
            String::new()
        };

        if i == selected {
            print!("  {CYAN}{BOLD}\u{276f}{RESET} {BOLD}{label}{RESET}{tag}");
        } else {
            print!("    {DIM}{label}{RESET}{tag}");
        }
    }

    // Footer
    print!("\x1b[B\r\x1b[2K  {DIM}\u{2191}/\u{2193} navigate  Enter select  Esc cancel{RESET}");

    // Move back up to top of reserved area (scroll-safe)
    print!("\x1b[{}A", total_lines - 1);

    io::stdout().flush().ok();
}
