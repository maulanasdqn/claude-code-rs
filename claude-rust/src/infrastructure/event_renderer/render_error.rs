use std::io::{self, Write};

use super::super::terminal::{BOLD, RED, RESET};
use super::super::terminal::term_width;

pub fn render_error_box(msg: &str) {
    let w = term_width().saturating_sub(4).max(24);
    let inner = w.saturating_sub(2);

    let top_label = "─ Error ";
    let top_fill = inner.saturating_sub(top_label.len());
    eprintln!();
    eprintln!("  {RED}{BOLD}╭{top_label}{}╮{RESET}", "─".repeat(top_fill));

    let max_content = inner.saturating_sub(4);
    let mut remaining = msg;
    while !remaining.is_empty() {
        let (line, rest) = if remaining.len() <= max_content {
            (remaining, "")
        } else {
            let split = remaining[..max_content.min(remaining.len())]
                .rfind(' ')
                .unwrap_or_else(|| max_content.min(remaining.len()));
            (&remaining[..split], remaining[split..].trim_start())
        };
        let pad = max_content.saturating_sub(line.len());
        eprintln!("  {RED}│{RESET}  {line}{}  {RED}│{RESET}", " ".repeat(pad));
        remaining = rest;
    }

    eprintln!("  {RED}{BOLD}╰{}╯{RESET}", "─".repeat(inner));
    eprintln!();
    io::stderr().flush().ok();
}
