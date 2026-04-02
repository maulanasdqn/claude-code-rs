use std::time::Duration;

use indicatif::ProgressBar;
use claude_rust_engine::EngineEvent;

use super::super::terminal::{BOLD, CYAN, DIM, MAGENTA, RESET, summarize_tool_input, tool_display_name};
use super::RenderState;
use super::render_error::render_error_box;
use super::render_md::{flush_line_buf, render_md_line};
use super::render_event_helpers::{
    clear_activity, ensure_activity, flush_last_read, fmt_tokens, random_verb,
    render_tool_result, save_json_to_last_tool, spinner_style, stop_thinking,
};

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
            if !state.in_text { state.in_text = true; state.text_started = false; }
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
            ensure_activity(state, format!("{display}…"));
            state.active_tools.push_back((name, String::new()));
        }

        EngineEvent::ToolInput { json_chunk } => {
            state.current_json_buf.push_str(&json_chunk);
            if let Some((tool_name, _)) = state.active_tools.back() {
                let partial = summarize_tool_input(tool_name, &state.current_json_buf);
                let display = tool_display_name(tool_name);
                let msg = if partial.is_empty() { format!("{display}…") } else { format!("{display}({partial})…") };
                if let Some(pb) = &state.activity_pb { pb.set_message(msg); }
            }
        }

        EngineEvent::ToolResult { name, output, is_error } => {
            save_json_to_last_tool(state);
            let (tool_name, json) = state.active_tools.pop_front()
                .unwrap_or_else(|| (name.clone(), String::new()));
            if state.active_tools.is_empty() {
                clear_activity(state);
            } else if let Some((next_name, _)) = state.active_tools.front() {
                let next_display = tool_display_name(next_name);
                if let Some(pb) = &state.activity_pb { pb.set_message(format!("{next_display}…")); }
            }
            render_tool_result(&tool_name, &json, &output, is_error, state);
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
                let cost = if cost_val < 0.0001 { "<$0.0001".to_string() }
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
