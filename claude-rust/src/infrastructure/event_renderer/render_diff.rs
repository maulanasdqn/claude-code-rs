use indicatif::MultiProgress;

use super::super::terminal::{DIM, GREEN, RED, RESET};

const MAX_DIFF_LINES: usize = 20;

pub(super) fn render_edit_diff(tool_json: &str, mp: &MultiProgress) {
    let v: serde_json::Value = match serde_json::from_str(tool_json) {
        Ok(v) => v,
        Err(_) => return,
    };
    let old = v["old_string"].as_str().unwrap_or("");
    let new = v["new_string"].as_str().unwrap_or("");
    if old.is_empty() && new.is_empty() {
        return;
    }

    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let show_old = old_lines.len().min(MAX_DIFF_LINES);
    for line in &old_lines[..show_old] {
        mp.println(format!("    {RED}{DIM}-{RESET} {DIM}{line}{RESET}")).ok();
    }
    if old_lines.len() > MAX_DIFF_LINES {
        mp.println(format!("    {DIM}  … {} more{RESET}", old_lines.len() - MAX_DIFF_LINES)).ok();
    }

    let show_new = new_lines.len().min(MAX_DIFF_LINES);
    for line in &new_lines[..show_new] {
        mp.println(format!("    {GREEN}{DIM}+{RESET} {DIM}{line}{RESET}")).ok();
    }
    if new_lines.len() > MAX_DIFF_LINES {
        mp.println(format!("    {DIM}  … {} more{RESET}", new_lines.len() - MAX_DIFF_LINES)).ok();
    }
}

pub(super) fn render_write_preview(tool_json: &str, mp: &MultiProgress) {
    let v: serde_json::Value = match serde_json::from_str(tool_json) {
        Ok(v) => v,
        Err(_) => return,
    };
    let content = v["content"].as_str().unwrap_or("");
    if content.is_empty() {
        return;
    }
    let lines: Vec<&str> = content.lines().collect();
    let show = lines.len().min(8);
    for line in &lines[..show] {
        mp.println(format!("    {DIM}│  {line}{RESET}")).ok();
    }
    if lines.len() > 8 {
        mp.println(format!("    {DIM}│  … {} more lines{RESET}", lines.len() - 8)).ok();
    }
}
