use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal;

use super::{BOLD, CYAN, DIM, GREEN, RESET};

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

    print!("\r\x1b[2K");
    for _ in 1..total_lines {
        print!("\x1b[B\r\x1b[2K");
    }

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

    print!("\r\x1b[2K");

    print!("\x1b[B\r\x1b[2K  {BOLD}{title}{RESET}");

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

    print!("\x1b[B\r\x1b[2K  {DIM}\u{2191}/\u{2193} navigate  Enter select  Esc cancel{RESET}");

    print!("\x1b[{}A", total_lines - 1);

    io::stdout().flush().ok();
}
