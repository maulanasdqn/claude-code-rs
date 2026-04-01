use std::io::{self, Write};

use crossterm::terminal;

pub fn pin_working_footer() {
    let (cols, rows) = match terminal::size() {
        Ok(s) => (s.0 as usize, s.1 as usize),
        Err(_) => return,
    };
    if rows < 6 {
        return;
    }
    let inner = cols.saturating_sub(4);
    let inner_dashes = "─".repeat(inner);
    print!("\x1b7");
    print!("\x1b[{};1H  \x1b[2m╭{}╮\x1b[0m", rows - 2, inner_dashes);
    print!(
        "\x1b[{};1H  \x1b[2m│\x1b[0m  ⟳  Working...\x1b[{};{}H\x1b[2m│\x1b[0m",
        rows - 1,
        rows - 1,
        cols - 2
    );
    print!("\x1b[{};1H  \x1b[2m╰{}╯\x1b[0m", rows, inner_dashes);
    print!("\x1b[1;{}r", rows - 3);
    print!("\x1b8");
    io::stdout().flush().ok();
}

pub fn unpin_working_footer() {
    print!("\x1b[r");
    let rows = terminal::size().map(|(_, r)| r as usize).unwrap_or(24);
    print!("\x1b[{};1H\x1b[J", rows - 2);
    io::stdout().flush().ok();
}
