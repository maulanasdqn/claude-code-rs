use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::{Arc, atomic::AtomicU8};

use crossterm::terminal;

use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RED, RESET, YELLOW};
use super::banner::layout_width;
use super::input_border::build_top_border;
use super::input_raw::read_line_raw;

pub fn read_user_input(mode: &Arc<AtomicU8>, history: &[String], skill_names: &[String]) -> Option<String> {
    if !io::stdin().is_terminal() {
        return read_user_input_simple();
    }

    terminal::disable_raw_mode().ok();

    let w = layout_width();
    let inner = w - 2;

    let top = build_top_border(inner, mode);
    let bot = format!("  {DIM}╰{}╯{RESET}", "─".repeat(inner));
    let avail = inner.saturating_sub(3);

    let pcolor = match PermissionMode::load(mode) {
        PermissionMode::Normal => CYAN,
        PermissionMode::Plan => MAGENTA,
        PermissionMode::AutoAccept => YELLOW,
        PermissionMode::Bypass => RED,
    };

    println!("{top}");
    println!("  {DIM}│{RESET} {BOLD}{pcolor}❯{RESET} {}{DIM}│{RESET}", " ".repeat(avail));
    println!("{bot}");

    print!("\x1b[2A\r\x1b[6C");
    io::stdout().flush().ok();

    terminal::enable_raw_mode().ok()?;
    let result = read_line_raw(inner, mode, history, skill_names);
    terminal::disable_raw_mode().ok();
    print!("\r");

    print!("\x1b[2B\r");
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
