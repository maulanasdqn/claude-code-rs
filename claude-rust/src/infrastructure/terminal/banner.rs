use crossterm::terminal;

use super::{BOLD, CYAN, DIM, RESET};

pub fn term_width() -> usize {
    terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

pub fn layout_width() -> usize {
    term_width().saturating_sub(4).max(10)
}

pub fn print_banner(cwd: &str) {
    let w = layout_width();
    let rule = "─".repeat(w);

    println!();
    println!("  {DIM}{rule}{RESET}");
    println!("  {BOLD}{CYAN}◆  Claude Code{RESET} {DIM}(Rust) v0.4.0{RESET}");
    println!("  {DIM}{rule}{RESET}");
    println!();

    let max_cwd = w.saturating_sub(5);
    let display_cwd = if cwd.len() > max_cwd {
        let keep = max_cwd.saturating_sub(3).max(1);
        format!("...{}", &cwd[cwd.len().saturating_sub(keep)..])
    } else {
        cwd.to_string()
    };
    println!("  {DIM}cwd:{RESET} {display_cwd}");
    println!("  {DIM}Type a message or{RESET} /help {DIM}for commands.{RESET}");
    println!();
}
