use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use claude_rust_tools::todo_store;
use super::super::terminal::{BOLD, DIM, GREEN, ORANGE, RED, RESET, summarize_tool_input, tool_display_name};
use super::RenderState;

pub(super) const TICKS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const THINK_VERBS: &[&str] = &[
    "Thinking", "Reasoning", "Pondering", "Analyzing",
    "Considering", "Processing", "Working", "Reflecting",
];

pub(super) fn random_verb() -> &'static str {
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0) % THINK_VERBS.len();
    THINK_VERBS[idx]
}

pub(super) fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}").unwrap().tick_strings(TICKS)
}

pub(super) fn ensure_activity(state: &mut RenderState, msg: String) {
    match &state.activity_pb {
        Some(pb) => pb.set_message(msg),
        None => {
            let pb = state.mp.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style());
            pb.set_message(msg);
            pb.enable_steady_tick(Duration::from_millis(150));
            state.activity_pb = Some(pb);
        }
    }
}

pub(super) fn clear_activity(state: &mut RenderState) {
    if let Some(pb) = state.activity_pb.take() { pb.finish_and_clear(); }
}

pub(super) fn stop_thinking(state: &mut RenderState) {
    if state.in_thinking {
        if let Some(pb) = state.thinking_pb.take() { pb.finish_and_clear(); }
        state.in_thinking = false;
    }
}

pub(super) fn save_json_to_last_tool(state: &mut RenderState) {
    if let Some(back) = state.active_tools.back_mut()
        && back.1.is_empty() && !state.current_json_buf.is_empty() {
            back.1 = std::mem::take(&mut state.current_json_buf);
        }
}

pub(super) fn fmt_arg(s: &str) -> String {
    if s.is_empty() { String::new() } else { format!("({s})") }
}

pub(super) fn fmt_tokens(n: u64) -> String {
    if n >= 1000 { format!("{:.1}K", n as f64 / 1000.0) } else { format!("{n}") }
}

pub(super) fn first_line(s: &str) -> &str { s.lines().next().unwrap_or(s) }

pub(super) fn flush_last_read(state: &mut RenderState) {
    let paths = std::mem::take(&mut state.last_read);
    if paths.is_empty() { return; }
    let n = paths.len();
    let noun = if n == 1 { "file" } else { "files" };
    let home = std::env::var("HOME").unwrap_or_default();
    state.mp.println(format!("  {BOLD}Reading {n} {noun}…{RESET}  {DIM}(ctrl+o to expand){RESET}")).ok();
    for (i, path) in paths.iter().enumerate() {
        let short = if !home.is_empty() && path.starts_with(&home) {
            format!("~{}", &path[home.len()..])
        } else { path.clone() };
        let conn = if i + 1 == n { "└" } else { "├" };
        state.mp.println(format!("  {DIM}{conn}  {short}{RESET}")).ok();
    }
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max { s.to_string() }
    else { format!("{}…", chars[..max - 1].iter().collect::<String>()) }
}

pub(super) fn render_tool_result(tool_name: &str, json: &str, output: &str, is_error: bool, state: &mut RenderState) {
    let summary = summarize_tool_input(tool_name, json);
    let display = tool_display_name(tool_name);
    let arg = fmt_arg(&summary);

    if is_error {
        state.mp.println(format!("  {DIM}{display}{arg}  {RED}✗  {}{RESET}", first_line(output))).ok();
        return;
    }

    match tool_name {
        "spawn_agent" => {
            let agent_id = serde_json::from_str::<serde_json::Value>(output)
                .ok().and_then(|v| v["agent_id"].as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let id_hint = if agent_id.is_empty() { String::new() } else { format!("  {DIM}→ {agent_id}{RESET}") };
            state.mp.println(format!("  {DIM}⟳  Spawn{arg}{RESET}{id_hint}")).ok();
        }
        "wait_agent" => {
            let (color, suffix) = serde_json::from_str::<serde_json::Value>(output).ok()
                .map(|v| {
                    let status = v["status"].as_str().unwrap_or("?");
                    if status == "completed" { (GREEN, "done".to_string()) }
                    else { (RED, format!("failed: {}", v["error"].as_str().unwrap_or("?"))) }
                })
                .unwrap_or((DIM, "done".to_string()));
            state.mp.println(format!("  {DIM}✓  Wait{arg}  {color}{suffix}{RESET}")).ok();
        }
        "list_agents" => {}
        "file_edit" => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
            super::render_diff::render_edit_diff(json, &state.mp);
        }
        "file_write" => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
            super::render_diff::render_write_preview(json, &state.mp);
        }
        "read" => {
            if !summary.is_empty() { state.last_read.push(summary); }
        }
        "glob" | "grep" | "web_fetch" | "web_search" => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
        }
        "agent" | "explore" => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
        }
        "todo_write" => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
            render_task_list(state);
        }
        _ => {
            state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
            if !output.is_empty() && output != "(no output)" {
                let lines: Vec<&str> = output.lines().collect();
                let show = lines.len().min(4);
                for line in &lines[..show] {
                    state.mp.println(format!("    {DIM}{}{RESET}", truncate(line, 120))).ok();
                }
                if lines.len() > show {
                    state.mp.println(format!("    {DIM}… {} more lines{RESET}", lines.len() - show)).ok();
                }
            }
        }
    }
}

fn render_task_list(state: &mut RenderState) {
    let todos = todo_store::read_todos();
    if todos.is_empty() { return; }
    if let Some(active_form) = todo_store::current_active_form() {
        if let Some(pb) = &state.thinking_pb { pb.set_message(format!("{active_form}…")); }
    }
    let count = todos.len();
    for (i, todo) in todos.iter().enumerate() {
        let (marker, color) = match todo.status.as_str() {
            "in_progress" => ("■", ORANGE),
            "completed"   => ("✓", GREEN),
            _             => ("□", DIM),
        };
        let connector = if i + 1 == count { "└─" } else { "├─" };
        let content = truncate(&todo.content, 80);
        state.mp.println(format!("    {DIM}{connector}{RESET} {color}{marker}{RESET}  {DIM}{content}{RESET}")).ok();
    }
}
