use std::io::{self, Write};
use std::sync::{Arc, atomic::AtomicU8};

use crossterm::cursor;
use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RESET, YELLOW};

pub(super) fn redraw_input_line(
    buf: &str,
    cursor_pos: usize,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    let prompt_cols = 5;
    let avail = inner_width.saturating_sub(prompt_cols + 1);

    let (visible, vis_cursor) = if buf.len() <= avail {
        (buf.to_string(), cursor_pos)
    } else {
        let start = if cursor_pos > avail { cursor_pos - avail } else { 0 };
        let end = (start + avail).min(buf.len());
        (buf[start..end].to_string(), cursor_pos - start)
    };

    let pad = avail.saturating_sub(visible.len());

    let pcolor = match PermissionMode::load(mode) {
        PermissionMode::Normal => CYAN,
        PermissionMode::Plan => MAGENTA,
        PermissionMode::AutoAccept => YELLOW,
    };

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
