use crossterm::terminal;

use super::{BOLD, CYAN, DIM, RESET};

pub fn term_width() -> usize {
    terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

pub fn layout_width() -> usize {
    term_width().saturating_sub(4).max(10)
}

pub fn print_banner(cwd: &str, model_id: &str) {
    let w = layout_width();

    println!();
    println!("  {BOLD}{CYAN}◆{RESET}  {BOLD}Claude Code{RESET}  {DIM}rust · v0.4.0{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(w));
    println!();

    let max_cwd = w.saturating_sub(2);
    let display_cwd = if cwd.len() > max_cwd {
        let keep = max_cwd.saturating_sub(3).max(1);
        format!("...{}", &cwd[cwd.len().saturating_sub(keep)..])
    } else {
        cwd.to_string()
    };

    let short_model = model_id.trim_start_matches("claude-");
    println!("  {DIM}cwd{RESET}  {display_cwd}");
    println!("  {DIM}model{RESET}  {DIM}{short_model}{RESET}");
    println!("  {DIM}/help{RESET} for commands  {DIM}·  Shift+Tab{RESET} to switch mode");
    println!();
}
