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

fn thinking_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}")
        .unwrap()
        .tick_strings(TICKS)
}

fn tool_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}")
        .unwrap()
        .tick_strings(TICKS)
}

fn new_tool_spinner(mp: &indicatif::MultiProgress, display: &str) -> ProgressBar {
    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(tool_style());
    pb.set_message(format!("{display}…"));
    pb.enable_steady_tick(Duration::from_millis(150));
    pb
}

pub fn render_event(event: EngineEvent, state: &mut RenderState) {
    match event {
        EngineEvent::ThinkingDelta(_) => {
            if !state.in_thinking {
                state.in_thinking = true;
                let pb = state.mp.add(ProgressBar::new_spinner());
                pb.set_style(thinking_style());
                pb.set_message(random_verb());
                pb.enable_steady_tick(Duration::from_millis(150));
                state.thinking_pb = Some(pb);
            }
        }

        EngineEvent::TextDelta(text) => {
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
            stop_thinking(state);
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
                state.text_started = false;
            }
            save_json_to_last_tool(state);
            state.current_json_buf.clear();

            let display = tool_display_name(&name);
            let pb = new_tool_spinner(&state.mp, &display);
            state.active_tools.push_back((pb, name, String::new()));
        }

        EngineEvent::ToolInput { json_chunk } => {
            state.current_json_buf.push_str(&json_chunk);
            // Update spinner with live partial arg preview
            if let Some((pb, tool_name, _)) = state.active_tools.back() {
                let partial = summarize_tool_input(tool_name, &state.current_json_buf);
                let display = tool_display_name(tool_name);
                let msg = if partial.is_empty() {
                    format!("{display}…")
                } else {
                    format!("{display}({partial})…")
                };
                pb.set_message(msg);
            }
        }

        EngineEvent::ToolResult { name, output, is_error } => {
            save_json_to_last_tool(state);

            let (pb, tool_name, json) = match state.active_tools.pop_front() {
                Some(t) => t,
                None => {
                    let pb = state.mp.add(ProgressBar::hidden());
                    (pb, name.clone(), String::new())
                }
            };

            pb.finish_and_clear();

            let summary = summarize_tool_input(&tool_name, &json);
            let display = tool_display_name(&tool_name);
            let arg = fmt_arg(&summary);

            if is_error {
                state.mp.println(format!(
                    "  {DIM}{display}{arg}  {RED}✗  {}{RESET}",
                    first_line(&output)
                )).ok();
            } else {
                match tool_name.as_str() {
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
                        let count = if n > 0 {
                            format!("  {DIM}({n} lines){RESET}")
                        } else {
                            String::new()
                        };
                        state.mp.println(format!(
                            "  {DIM}{display}{arg}{RESET}{count}"
                        )).ok();
                    }
                    "todo_write" => {
                        state.mp.println(format!("  {DIM}{display}{arg}{RESET}")).ok();
                        render_task_list(state);
                    }
                    _ => {
                        state.mp.println(format!(
                            "  {DIM}{display}{arg}{RESET}"
                        )).ok();
                        if !output.is_empty() && output != "(no output)" {
                            let lines: Vec<&str> = output.lines().collect();
                            let show = lines.len().min(4);
                            for line in &lines[..show] {
                                let s = truncate(line, 120);
                                state.mp.println(format!("    {DIM}{s}{RESET}")).ok();
                            }
                            if lines.len() > show {
                                state.mp.println(format!(
                                    "    {DIM}… {} more lines{RESET}",
                                    lines.len() - show
                                )).ok();
                            }
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
            stop_thinking(state);
            if state.in_text { flush_line_buf(state); state.in_text = false; state.text_started = false; }
            if state.turn_input > 0 || state.turn_output > 0 {
                let i = fmt_tokens(state.turn_input);
                let o = fmt_tokens(state.turn_output);
                let cost_val = state.turn_input as f64 * 3.0 / 1_000_000.0
                    + state.turn_output as f64 * 15.0 / 1_000_000.0;
                let cost = if cost_val < 0.0001 {
                    format!("<$0.0001")
                } else if cost_val > 0.50 {
                    format!("${cost_val:.2}")
                } else {
                    format!("${cost_val:.4}")
                };
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
    if todos.is_empty() {
        return;
    }

    // Update the thinking spinner with the active task if available
    if let Some(active_form) = todo_store::current_active_form() {
        if let Some(pb) = &state.thinking_pb {
            pb.set_message(format!("{active_form}…"));
        }
    }

    // Print task list as plain scrollable text — no progress bars to avoid duplication
    let count = todos.len();
    for (i, todo) in todos.iter().enumerate() {
        let (marker, color) = match todo.status.as_str() {
            "in_progress" => ("■", ORANGE),
            "completed" => ("✓", GREEN),
            _ => ("□", DIM),
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
        && back.2.is_empty() && !state.current_json_buf.is_empty() {
            back.2 = std::mem::take(&mut state.current_json_buf);
        }
}

/// Format tool argument as `(arg)` or empty string if no arg.
fn fmt_arg(s: &str) -> String {
    if s.is_empty() { String::new() } else { format!("({s})") }
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1000 { format!("{:.1}K", n as f64 / 1000.0) } else { format!("{n}") }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max - 1].iter().collect::<String>())
    }
}
