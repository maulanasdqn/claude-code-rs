use std::io::{self, Write};
use std::sync::{Arc, atomic::AtomicU8};

use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RESET, YELLOW};
use super::git::git_branch;

pub(super) fn build_top_border(inner: usize, mode: &Arc<AtomicU8>) -> String {
    let mode_val = PermissionMode::load(mode);
    let branch = git_branch();

    let badge = match mode_val {
        PermissionMode::Normal => "",
        PermissionMode::Plan => " PLAN ",
        PermissionMode::AutoAccept => " AUTO ",
    };
    let badge_color = match mode_val {
        PermissionMode::Normal => CYAN,
        PermissionMode::Plan => MAGENTA,
        PermissionMode::AutoAccept => YELLOW,
    };

    let branch_vis = branch
        .as_deref()
        .filter(|b| badge.len() + b.len() + 7 <= inner)
        .map(|b| b.len() + 5);

    let total_fixed = badge.len() + branch_vis.unwrap_or(0);
    let dash_count = inner.saturating_sub(total_fixed);

    let badge_str = if badge.is_empty() {
        String::new()
    } else {
        format!("{badge_color}{BOLD}{badge}{RESET}{DIM}")
    };

    let dashes = "─".repeat(dash_count);

    let branch_str = branch
        .as_deref()
        .filter(|_| branch_vis.is_some())
        .map(|b| format!("{RESET} on {CYAN}{b}{RESET}{DIM} "))
        .unwrap_or_default();

    format!("  {DIM}╭{badge_str}{dashes}{branch_str}╮{RESET}")
}

pub(super) fn redraw_top_border(inner_width: usize, mode: &Arc<AtomicU8>) {
    let top = build_top_border(inner_width, mode);
    print!("\x1b7");
    print!("\x1b[1A\r");
    print!("{top}\x1b[K");
    print!("\x1b8");
    io::stdout().flush().ok();
}
