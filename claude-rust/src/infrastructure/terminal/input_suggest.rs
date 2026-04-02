use std::io::{self, Write};

use super::{BOLD, CYAN, DIM, RESET};

pub(super) const COMMAND_CATALOG: &[(&str, &str)] = &[
    ("/help", "Show available commands"),
    ("/clear", "Clear conversation history"),
    ("/compact", "Compact context to save tokens"),
    ("/cost", "Show token usage and cost"),
    ("/usage", "Show plan usage limits"),
    ("/model", "Switch or show current model"),
    ("/fast", "Toggle fast (haiku) model"),
    ("/effort", "Set effort level"),
    ("/mode", "Cycle permission mode"),
    ("/plan", "Toggle plan mode"),
    ("/conductor", "Run multi-agent orchestration"),
    ("/reflect", "Toggle self-correction pass"),
    ("/think", "Toggle extended thinking"),
    ("/diff", "Show git diff"),
    ("/status", "Show git status"),
    ("/review", "Review git diff"),
    ("/commit", "Generate commit message"),
    ("/doctor", "Check environment health"),
    ("/config", "Show merged settings"),
    ("/permissions", "Show allow/deny rules"),
    ("/session", "List or load sessions"),
    ("/memory", "Show CLAUDE.md"),
    ("/export", "Export conversation"),
    ("/rewind", "Remove last n exchanges"),
    ("/init", "Create/update CLAUDE.md"),
    ("/add", "Pin file to context"),
    ("/files", "Show pinned files"),
    ("/skills", "List available skills"),
    ("/copy", "Copy last response"),
    ("/login", "Authentication instructions"),
    ("/logout", "Clear credentials"),
    ("/vim", "Show vim mode status"),
    ("/version", "Show version"),
    ("/undo", "Restore file edits"),
    ("/quit", "Exit session"),
];

const MAX_VISIBLE: usize = 8;

pub(super) fn get_suggestions(
    input: &str,
    skill_names: &[(String, String)],
) -> Vec<(String, String)> {
    if !input.starts_with('/') || input.contains(' ') {
        return Vec::new();
    }

    let query = input.to_lowercase();

    let mut matches: Vec<(String, String)> = COMMAND_CATALOG
        .iter()
        .filter(|(cmd, _)| cmd.starts_with(&query))
        .map(|(cmd, desc)| (cmd.to_string(), desc.to_string()))
        .collect();

    for (name, desc) in skill_names {
        let full = format!("/{name}");
        if full.to_lowercase().starts_with(&query) {
            let label = if desc.is_empty() { "Skill".to_string() } else { desc.clone() };
            matches.push((full, label));
        }
    }

    matches
}

pub(super) fn render_suggestions(
    suggestions: &[(String, String)],
    selected_idx: Option<usize>,
    inner_width: usize,
) {
    if suggestions.is_empty() {
        return;
    }

    let count = suggestions.len().min(MAX_VISIBLE);
    let avail = inner_width.saturating_sub(3);
    let has_more = suggestions.len() > MAX_VISIBLE;
    let extra = if has_more { 1 } else { 0 };
    let total_lines = count + extra + 1; // suggestions + optional "more" + bottom border

    // Reserve space below by printing newlines (handles terminal scrolling correctly).
    // Even if at the bottom of the screen, the \n scrolls and the subsequent
    // relative \x1b[A movement always returns to the correct content line.
    for _ in 0..total_lines {
        println!();
    }
    // Move back up to content line using relative movement (scroll-safe)
    print!("\x1b[{}A", total_lines);

    // Draw each suggestion line (cursor-down won't scroll since space was reserved)
    for (i, (cmd, desc)) in suggestions.iter().enumerate().take(count) {
        print!("\x1b[B\r\x1b[2K"); // Move down, clear line
        let is_selected = selected_idx == Some(i);

        let cmd_part = format!("{cmd}");
        let desc_part = format!("  {desc}");
        let line_len = cmd_part.len() + desc_part.len();
        let truncated_desc = if line_len > avail {
            let spare = avail.saturating_sub(cmd_part.len() + 2);
            if spare > 3 {
                format!("  {}…", &desc[..spare.saturating_sub(1).min(desc.len())])
            } else {
                String::new()
            }
        } else {
            desc_part
        };

        if is_selected {
            print!("  {CYAN}{BOLD}{cmd_part}{RESET}{DIM}{truncated_desc}{RESET}");
        } else {
            print!("  {DIM}{cmd_part}{truncated_desc}{RESET}");
        }
    }

    if has_more {
        print!("\x1b[B\r\x1b[2K"); // Move down, clear line
        let more = suggestions.len() - MAX_VISIBLE;
        print!("  {DIM}  … +{more} more{RESET}");
    }

    // Bottom separator (flat, matching input box style)
    print!("\x1b[B\r\x1b[2K"); // Move down, clear line
    print!("  {DIM}{}{RESET}", "─".repeat(inner_width));

    // Move back up to content line using relative movement (scroll-safe)
    print!("\x1b[{}A", total_lines);

    io::stdout().flush().ok();
}

pub(super) fn clear_suggestions(prev_count: usize, inner_width: usize) {
    if prev_count == 0 {
        return;
    }

    let visible = prev_count.min(MAX_VISIBLE);
    let has_more = prev_count > MAX_VISIBLE;
    let extra = if has_more { 1 } else { 0 };
    let total_lines = visible + extra + 1; // +1 for bottom border

    // Move down and clear each line (space exists from previous render)
    for _ in 0..total_lines {
        print!("\x1b[B\r\x1b[2K");
    }

    // Move back up to content line
    print!("\x1b[{}A", total_lines);

    // Redraw the original input bottom border (1 line below content)
    print!("\x1b[B\r\x1b[2K  {DIM}{}{RESET}", "─".repeat(inner_width));
    print!("\x1b[1A"); // Back to content line

    io::stdout().flush().ok();
}
