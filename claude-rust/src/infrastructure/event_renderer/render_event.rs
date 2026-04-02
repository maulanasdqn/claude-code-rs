use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use claude_rust_engine::EngineEvent;

use claude_rust_tools::todo_store;
use super::super::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, ORANGE, RED, RESET, summarize_tool_input, tool_display_name};
use super::RenderState;
use super::render_error::render_error_box;
use super::render_md::{flush_line_buf, render_md_line};

const TICKS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const THINK_VERBS: &[&str] = &[
    "Thinking", "Reasoning", "Pondering", "Analyzing",
    "Considering", "Processing", "Working", "Reflecting",
];

fn random_verb() -> &'static str {
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % THINK_VERBS.len();
    THINK_VERBS[idx]
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}")
        .unwrap()
        .tick_strings(TICKS)
}

/// Ensure there is exactly one activity spinner, set its message, return reference.
fn ensure_activity(state: &mut RenderState, msg: String) {
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

/// Clear the single activity spinner if present.
fn clear_activity(state: &mut RenderState) {
    if let Some(pb) = state.activity_pb.take() {
        pb.finish_and_clear();
    }
}

pub fn render_event(event: EngineEvent, state: &mut RenderState) {
    match event {
        EngineEvent::ThinkingDelta(_) => {
            if !state.in_thinking {
                state.in_thinking = true;
                let pb = state.mp.add(ProgressBar::new_spinner());
                pb.set_style(spinner_style());
                pb.set_message(random_verb());
                pb.enable_steady_tick(Duration::from_millis(150));
                state.thinking_pb = Some(pb);
            }
        }

        EngineEvent::TextDelta(text) => {
            flush_last_read(state);
            clear_activity(state);
            stop_thinking(state);
            if !state.in_text {
                state.in_text = true;
                state.text_started = false;
            }
            state.line_buf.push_str(&text);
            while let Some(pos) = state.line_buf.find('\n') {
                let line = state.line_buf[..pos].to_string();
                state.line_buf.drain(..=pos);
                if !state.text_started {
                    state.text_started = true;
                    state.mp.println(format!("\n  {DIM}●{RESET}")).ok();
                }
                render_md_line(&line, state);
            }
        }

        EngineEvent::ToolStart { name, .. } => {
            // Flush dedup buffer when a non-read tool starts
            if !matches!(name.as_str(), "read" | "glob" | "grep" | "web_fetch" | "web_search") {
                flush_last_read(state);
            }
            stop_thinking(state);
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
                state.text_started = false;
            }
            save_json_to_last_tool(state);
            state.current_json_buf.clear();

            let display = tool_display_name(&name);
            // Update the single shared spinner — never create a second one.
            ensure_activity(state, format!("{display}…"));
            state.active_tools.push_back((name, String::new()));
        }

        EngineEvent::ToolInput { json_chunk } => {
            state.current_json_buf.push_str(&json_chunk);
            // Update the single spinner with a live partial-arg preview
            if let Some((tool_name, _)) = state.active_tools.back() {
                let partial = summarize_tool_input(tool_name, &state.current_json_buf);
                let display = tool_display_name(tool_name);
                let msg = if partial.is_empty() {
                    format!("{display}…")
                } else {
                    format!("{display}({partial})…")
                };
                if let Some(pb) = &state.activity_pb {
                    pb.set_message(msg);
                }
            }
        }

        EngineEvent::ToolResult { name, output, is_error } => {
            save_json_to_last_tool(state);

            let (tool_name, json) = state.active_tools.pop_front()
                .unwrap_or_else(|| (name.clone(), String::new()));

            // If no more pending tools, clear the activity spinner.
            // Otherwise keep it alive and update to the next tool's name.
            if state.active_tools.is_empty() {
                clear_activity(state);
            } else if let Some((next_name, _)) = state.active_tools.front() {
                let next_display = tool_display_name(next_name);
                if let Some(pb) = &state.activity_pb {
                    pb.set_message(format!("{next_display}…"));
                }
            }

            let summary = summarize_tool_input(&tool_name, &json);
            let display = tool_display_name(&tool_name);
            let arg = fmt_arg(&summary);

            if is_error {
                state.mp.println(format!(
                    "  {DIM}{display}{arg}  {RED}✗  {}{RESET}",
                    first_line(&output)
                )).ok();
                return;
            }

            match tool_name.as_str() {
                // Conductor tools: compact single-line output, no output dump
                "spawn_agent" => {
                    let agent_id = serde_json::from_str::<serde_json::Value>(&output)
                        .ok()
                        .and_then(|v| v["agent_id"].as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    let id_hint = if agent_id.is_empty() { String::new() }
                        else { format!("  {DIM}→ {agent_id}{RESET}") };
                    state.mp.println(format!("  {DIM}⟳  Spawn{arg}{RESET}{id_hint}")).ok();
                }
                "wait_agent" => {
                    let (color, suffix) = serde_json::from_str::<serde_json::Value>(&output)
                        .ok()
                        .map(|v| {
                            let status = v["status"].as_str().unwrap_or("?");
                            if status == "completed" { (GREEN, "done".to_string()) }
                            else { (RED, format!("failed: {}", v["error"].as_str().unwrap_or("?"))) }
                        })
                        .unwrap_or((DIM, "done".to_string()));
                    state.mp.println(format!("  {DIM}✓  Wait{arg}  {color}{suffix}{RESET}")).ok();
                }
                "list_agents" => {
                    // Suppress — internal conductor bookkeeping
                }
                "file_edit" => {
                    state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
                    super::render_diff::render_edit_diff(&json, &state.mp);
                }
                "file_write" => {
                    state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
                    super::render_diff::render_write_preview(&json, &state.mp);
                }
                "read" | "glob" | "grep" | "web_fetch" | "web_search" => {
                    let n = output.lines().count();
                    let label = format!("{display}{arg}");
                    // Deduplicate consecutive reads to the same target
                    match &mut state.last_read {
                        Some((prev, count, prev_n)) if *prev == label => {
                            *count += 1;
                            *prev_n = n;
                        }
                        _ => {
                            flush_last_read(state);
                            state.last_read = Some((label, 1, n));
                        }
                    }
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
                            state.mp.println(format!(
                                "    {DIM}… {} more lines{RESET}", lines.len() - show
                            )).ok();
                        }
                    }
                }
            }
            let _ = name;
        }

        EngineEvent::Usage { input_tokens, output_tokens } => {
            state.turn_input += input_tokens;
            state.turn_output += output_tokens;
        }

        EngineEvent::HookOutput { source, output } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            for line in output.lines() {
                state.mp.println(format!("  {DIM}[hook:{source}] {line}{RESET}")).ok();
            }
        }

        EngineEvent::Compacted { original_turns } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            state.mp.println(format!("\n  {DIM}{MAGENTA}◆  compacted — {original_turns} messages summarized{RESET}")).ok();
        }

        EngineEvent::ModeChanged { mode } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            state.mp.println(format!("\n  {CYAN}{BOLD}◆  {}{RESET}  {DIM}{}{RESET}", mode.label(), mode.description())).ok();
        }

        EngineEvent::TurnComplete => {
            flush_last_read(state);
            clear_activity(state);
            stop_thinking(state);
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            if state.turn_input > 0 || state.turn_output > 0 {
                let i = fmt_tokens(state.turn_input);
                let o = fmt_tokens(state.turn_output);
                let cost_val = state.turn_input as f64 * 3.0 / 1_000_000.0
                    + state.turn_output as f64 * 15.0 / 1_000_000.0;
                let cost = if cost_val < 0.0001 { format!("<$0.0001") }
                    else if cost_val > 0.50 { format!("${cost_val:.2}") }
                    else { format!("${cost_val:.4}") };
                state.mp.println(format!("\n  {DIM}∙  {i} in  ·  {o} out  ·  {cost}{RESET}")).ok();
                state.turn_input = 0;
                state.turn_output = 0;
            }
            state.mp.println(String::new()).ok();
        }

        EngineEvent::Error(msg) => {
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            render_error_box(&msg);
        }
    }
}

fn render_task_list(state: &mut RenderState) {
    let todos = todo_store::read_todos();
    if todos.is_empty() { return; }

    if let Some(active_form) = todo_store::current_active_form() {
        if let Some(pb) = &state.thinking_pb {
            pb.set_message(format!("{active_form}…"));
        }
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

fn stop_thinking(state: &mut RenderState) {
    if state.in_thinking {
        if let Some(pb) = state.thinking_pb.take() {
            pb.finish_and_clear();
        }
        state.in_thinking = false;
    }
}

fn save_json_to_last_tool(state: &mut RenderState) {
    if let Some(back) = state.active_tools.back_mut()
        && back.1.is_empty() && !state.current_json_buf.is_empty() {
            back.1 = std::mem::take(&mut state.current_json_buf);
        }
}

fn fmt_arg(s: &str) -> String {
    if s.is_empty() { String::new() } else { format!("({s})") }
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1000 { format!("{:.1}K", n as f64 / 1000.0) } else { format!("{n}") }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

/// Flush the dedup buffer — print the collapsed read/search line.
fn flush_last_read(state: &mut RenderState) {
    if let Some((label, count, lines)) = state.last_read.take() {
        let count_tag = if count > 1 { format!("  {DIM}×{count}{RESET}") } else { String::new() };
        let lines_tag = if lines > 0 { format!("  {DIM}({lines} lines){RESET}") } else { String::new() };
        state.mp.println(format!("  {DIM}{label}{RESET}{count_tag}{lines_tag}")).ok();
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max { s.to_string() }
    else { format!("{}…", chars[..max - 1].iter().collect::<String>()) }
}
