use std::io::{self, Write};
use std::sync::{Arc, atomic::AtomicU8};

use crossterm::cursor;
use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RED, RESET, YELLOW};

pub(super) fn redraw_input(
    buf: &str,
    cursor_pos: usize,
    extra_lines: usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    if extra_lines == 0 {
        redraw_input_line(buf, cursor_pos, inner_width, mode);
    } else {
        redraw_full_input(buf, cursor_pos, extra_lines, inner_width, mode);
    }
}

pub(super) fn redraw_input_line(
    buf: &str,
    cursor_pos: usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    let avail = inner_width.saturating_sub(3);

    let (visible, vis_cursor) = if buf.len() <= avail {
        (buf.to_string(), cursor_pos)
    } else {
        let start = if cursor_pos > avail { cursor_pos - avail } else { 0 };
        let end = (start + avail).min(buf.len());
        (buf[start..end].to_string(), cursor_pos - start)
    };

    let pad = avail.saturating_sub(visible.len());

    let pcolor = pcolor(mode);

    print!(
        "\r  {DIM}│{RESET} {BOLD}{pcolor}❯{RESET} {visible}{}{DIM}│{RESET}",
        " ".repeat(pad)
    );

    let cursor_col = 6 + vis_cursor;
    print!("\r");
    if cursor_col > 0 {
        print!("{}", cursor::MoveRight(cursor_col as u16));
    }
    io::stdout().flush().ok();
}

fn redraw_full_input(
    buf: &str,
    cursor_pos: usize,
    extra_lines: usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    let avail = inner_width.saturating_sub(3);
    let cursor_row = buf[..cursor_pos].chars().filter(|&c| c == '\n').count();

    if cursor_row > 0 {
        print!("\x1b[{}A", cursor_row);
    }

    let content_lines: Vec<&str> = buf.split('\n').collect();
    for (i, line) in content_lines.iter().enumerate() {
        let pad = avail.saturating_sub(line.len());
        print!("\r\x1b[2K");
        if i == 0 {
            let pc = pcolor(mode);
            print!("  {DIM}│{RESET} {BOLD}{pc}❯{RESET} {line}{}{DIM}│{RESET}", " ".repeat(pad));
        } else {
            print!("  {DIM}│{RESET}   {line}{}{DIM}│{RESET}", " ".repeat(pad + 2));
        }
        println!();
    }

    let bot = format!("  {DIM}╰{}╯{RESET}", "─".repeat(inner_width));
    print!("\r\x1b[2K{bot}");

    let rows_up = extra_lines - cursor_row + 1;
    if rows_up > 0 {
        print!("\x1b[{}A", rows_up);
    }

    let newline_before = buf[..cursor_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col_in_line = cursor_pos - newline_before;
    let col = if cursor_row == 0 { 6 + col_in_line } else { 4 + col_in_line };
    print!("\r");
    if col > 0 {
        print!("{}", cursor::MoveRight(col as u16));
    }
    io::stdout().flush().ok();
}

fn pcolor(mode: &Arc<AtomicU8>) -> &'static str {
    match PermissionMode::load(mode) {
        PermissionMode::Normal => CYAN,
        PermissionMode::Plan => MAGENTA,
        PermissionMode::AutoAccept => YELLOW,
        PermissionMode::Bypass => RED,
    }
}
