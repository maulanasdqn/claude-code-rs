use std::io::{self, Write};

use claude_rust_engine::EngineEvent;

use super::super::terminal::{BOLD, DIM, MAGENTA, RESET, YELLOW, summarize_tool_input, tool_icon};
use super::RenderState;
use super::render_error::render_error_box;
use super::render_md::{flush_line_buf, render_md_line};

pub fn render_event(event: EngineEvent, state: &mut RenderState) {
    match event {
        EngineEvent::ThinkingDelta(_) => {
            if !state.in_thinking {
                print!("\n  {DIM}💭 Let me think...{RESET}");
                io::stdout().flush().ok();
                state.in_thinking = true;
            }
        }

        EngineEvent::TextDelta(text) => {
            if state.in_thinking {
                print!("\r\x1b[2K");
                state.in_thinking = false;
            }
            if !state.in_text {
                println!();
                state.in_text = true;
            }
            state.line_buf.push_str(&text);
            while let Some(pos) = state.line_buf.find('\n') {
                let line = state.line_buf[..pos].to_string();
                state.line_buf.drain(..=pos);
                render_md_line(&line);
            }
            io::stdout().flush().ok();
        }

        EngineEvent::ToolStart { name, .. } => {
            if state.in_thinking {
                print!("\r\x1b[2K");
                state.in_thinking = false;
            }
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
            }
            state.tool_json_buf.clear();
            state.current_tool_name = name.clone();

            let icon = tool_icon(&name);
            println!("\n  {DIM}{icon}{RESET} {YELLOW}{BOLD}{name}{RESET}");
            io::stdout().flush().ok();
        }

        EngineEvent::ToolInput { json_chunk } => {
            state.tool_json_buf.push_str(&json_chunk);
        }

        EngineEvent::ToolResult {
            name,
            output,
            is_error,
        } => {
            use super::super::terminal::{ITALIC, RED};
            let summary = summarize_tool_input(&state.current_tool_name, &state.tool_json_buf);
            if !summary.is_empty() {
                println!("    {DIM}› {ITALIC}{summary}{RESET}");
            }
            state.tool_json_buf.clear();

            if is_error {
                println!("    {RED}✗ {output}{RESET}");
            } else if output.is_empty() || output == "(no output)" {
                println!("    {DIM}(no output){RESET}");
            } else {
                let max_lines = 20;
                let max_chars = 1000;
                let preview = if output.len() > max_chars {
                    format!("{}…", &output[..max_chars])
                } else {
                    output.clone()
                };
                let lines: Vec<&str> = preview.lines().collect();
                let show = lines.len().min(max_lines);
                println!("    {DIM}┄┄┄{RESET}");
                for line in &lines[..show] {
                    println!("    {DIM}{line}{RESET}");
                }
                if lines.len() > max_lines {
                    let remaining = lines.len() - max_lines;
                    println!("    {DIM}… {remaining} more lines{RESET}");
                }
            }
            let _ = name;
        }

        EngineEvent::Usage { .. } => {}

        EngineEvent::Compacted { original_turns } => {
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
            }
            println!();
            println!(
                "  {DIM}{MAGENTA}⟳  Context compacted — {original_turns} messages summarized{RESET}"
            );
        }

        EngineEvent::ModeChanged { mode } => {
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
            }
            println!(
                "\n  {MAGENTA}{BOLD}● {}{RESET} {DIM}— {}{RESET}",
                mode.label(),
                mode.description()
            );
        }

        EngineEvent::TurnComplete => {
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
            }
            if state.in_thinking {
                print!("\r\x1b[2K");
                state.in_thinking = false;
            }
            println!();
        }

        EngineEvent::Error(msg) => {
            if state.in_text {
                flush_line_buf(state);
                state.in_text = false;
            }
            render_error_box(&msg);
        }
    }
}
