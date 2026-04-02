use std::io::{self, Write};
use std::sync::{Arc, Mutex, OnceLock, atomic::AtomicU8};

use claude_rust_types::PermissionMode;

use super::{BOLD, CYAN, DIM, MAGENTA, RED, RESET, YELLOW};
use super::git::git_branch;

static CURRENT_MODEL: OnceLock<Mutex<String>> = OnceLock::new();

fn model_store() -> &'static Mutex<String> {
    CURRENT_MODEL.get_or_init(|| Mutex::new(String::new()))
}

pub fn set_current_model(model: &str) {
    let short = model.trim_start_matches("claude-");
    if let Ok(mut g) = model_store().lock() {
        *g = short.to_string();
    }
}

fn current_model_short() -> String {
    model_store().lock().map(|g| g.clone()).unwrap_or_default()
}

/// Separator line with branch name right-aligned, no box corners.
pub(super) fn build_top_border(inner: usize, _mode: &Arc<AtomicU8>) -> String {
    let branch = git_branch();
    match branch.as_deref() {
        Some(b) if b.len() + 4 <= inner => {
            let left = inner.saturating_sub(b.len() + 4);
            format!("  {DIM}{}{RESET} {BOLD}{CYAN}{b}{RESET}{DIM} ──{RESET}", "─".repeat(left))
        }
        _ => format!("  {DIM}{}{RESET}", "─".repeat(inner)),
    }
}

/// Status bar shown below the input prompt.
pub(super) fn build_status_bar(mode: &Arc<AtomicU8>) -> String {
    let model = current_model_short();
    let model_tag = if model.is_empty() {
        String::new()
    } else {
        format!("  {DIM}[{model}]{RESET}")
    };
    let hint = format!("{DIM}shift+tab: mode · esc: interrupt · ctrl+g: editor{RESET}");
    match PermissionMode::load(mode) {
        PermissionMode::Normal => {
            format!("  {DIM}▶▶ default mode{RESET}{model_tag}  {hint}")
        }
        PermissionMode::Plan => {
            format!("  {MAGENTA}{BOLD}▶▶{RESET} {MAGENTA}plan mode{RESET}{model_tag}  {hint}")
        }
        PermissionMode::AutoAccept => {
            format!("  {YELLOW}{BOLD}▶▶{RESET} {YELLOW}auto-accept{RESET}{model_tag}  {hint}")
        }
        PermissionMode::Bypass => {
            format!("  {RED}{BOLD}▶▶{RESET} {RED}bypass permissions{RESET}{model_tag}  {hint}")
        }
    }
}

/// Redraw status bar in-place after a mode change.
pub(super) fn redraw_status_bar(mode: &Arc<AtomicU8>) {
    let bar = build_status_bar(mode);
    print!("\x1b7");        // save cursor
    print!("\x1b[2B\r");   // move down 2 (past bottom border to status bar)
    print!("{bar}\x1b[K"); // redraw + clear rest of line
    print!("\x1b8");       // restore cursor
    io::stdout().flush().ok();
}
