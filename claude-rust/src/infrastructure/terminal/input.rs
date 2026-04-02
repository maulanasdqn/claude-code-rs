use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::{Arc, atomic::AtomicU8};

use crossterm::{cursor, terminal};

use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RED, RESET, YELLOW};
use super::banner::layout_width;
use super::input_border::{build_status_bar, build_top_border};
use super::input_raw::read_line_raw;

pub fn read_user_input(mode: &Arc<AtomicU8>, history: &[String], skill_names: &[(String, String)]) -> Option<String> {
    if !io::stdin().is_terminal() {
        return read_user_input_simple();
    }

    terminal::disable_raw_mode().ok();

    // Ensure clean terminal state before drawing input box
    print!("\x1b[0m\r");
    io::stdout().flush().ok();

    // Pin input box to the bottom of the visible terminal area.
    // Query cursor position and terminal height; print newlines to push
    // the 4-line box (top border + input + bottom border + status bar)
    // to the very bottom so output scrolls above it.
    if let (Ok((_, cur_row)), Ok((_, rows))) = (cursor::position(), terminal::size()) {
        let cur_row = cur_row as usize;
        let rows = rows as usize;
        // We need 4 lines; target the box to start at rows-5 (leaves 1 spare)
        let target = rows.saturating_sub(5);
        if cur_row < target {
            print!("{}", "\n".repeat(target - cur_row));
            io::stdout().flush().ok();
        }
    }

    let w = layout_width();
    let inner = w.saturating_sub(2);

    let top = build_top_border(inner, mode);
    let bot = format!("  {DIM}{}{RESET}", "─".repeat(inner));
    let bar = build_status_bar(mode);
    // avail: how many text chars fit on one line after "  ❯ " (4 cols)
    let avail = inner.saturating_sub(1);

    let pcolor = match PermissionMode::load(mode) {
        PermissionMode::Normal => CYAN,
        PermissionMode::Plan => MAGENTA,
        PermissionMode::AutoAccept => YELLOW,
        PermissionMode::Bypass => RED,
    };

    // Layout: top | input | bottom | status bar  (4 lines)
    print!("\x1b[2K"); println!("{top}");
    print!("\x1b[2K"); println!("  {BOLD}{pcolor}❯{RESET} {}", " ".repeat(avail));
    print!("\x1b[2K"); println!("{bot}");
    print!("\x1b[2K"); println!("{bar}");

    // Move cursor back to the input line (3 up), position after "  ❯ " (col 4)
    print!("\x1b[3A\r\x1b[4C");
    io::stdout().flush().ok();

    terminal::enable_raw_mode().ok()?;
    let (result, extra_lines) = read_line_raw(inner, mode, history, skill_names);
    terminal::disable_raw_mode().ok();

    // Clear the entire input box.
    // Cursor is currently on the last content line of the input box.
    // The box has: top_border + (extra_lines+1) content lines + bot_border + status_bar
    // = extra_lines + 3 lines above and below the last content line.
    // Move up to top border (extra_lines + 1 lines up), then clear to end of screen.
    let lines_to_top = extra_lines + 1;
    print!("\x1b[{}A\r\x1b[J", lines_to_top);

    // For normal text submissions, print the question as a clean single line.
    // Skip special signals (\x00 prefix) and empty strings.
    if let Some(ref text) = result {
        if !text.is_empty() && !text.starts_with('\x00') {
            let display = if let Some(first) = text.lines().next() {
                if text.contains('\n') { format!("{first}…") } else { first.to_string() }
            } else {
                text.clone()
            };
            println!("  {BOLD}{pcolor}❯{RESET}  {DIM}{display}{RESET}");
            println!();
        }
    }

    io::stdout().flush().ok();
    result
}

pub fn prompt_resume() -> bool {
    eprint!("  {DIM}Previous session found. Resume? {RESET}{BOLD}[y/n]{RESET} ");
    io::stderr().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        return line.trim().starts_with('y');
    }
    false
}

pub(super) fn read_user_input_simple() -> Option<String> {
    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                Some(String::new())
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}
