use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use claude_rust_engine::EngineEvent;

use super::super::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, ORANGE, RESET, summarize_tool_input, tool_display_name, tool_icon};
use super::RenderState;
use super::render_error::render_error_box;
use super::render_md::{flush_line_buf, render_md_line};
use super::render_spinner::SpinnerState;

pub fn render_event(event: EngineEvent, state: &mut RenderState) {
    match event {
        EngineEvent::ThinkingDelta(_) => {
            if !state.in_thinking {
                println!();
                state.in_thinking = true;
                state.spinner = Some(SpinnerState::new());
                state.spin_frame = 0;
                state.thinking_start = Some(std::time::Instant::now());
            }
            state.spin_frame += 1;
            let line = state.spinner.as_ref().map(|s| s.render_frame(state.spin_frame)).unwrap_or_default();
            print!("\r{line}\x1b[K");
            io::stdout().flush().ok();
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
                render_md_line(&line, state);
            }
            io::stdout().flush().ok();
        }

        EngineEvent::ToolStart { name, .. } => {
            if state.in_thinking { print!("\r\x1b[2K"); state.in_thinking = false; state.spinner = None; }
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            state.tool_json_buf.clear();
            state.current_tool_name = name.clone();
            let icon = tool_icon(&name);
            let display = tool_display_name(&name);
            let active = Arc::new(AtomicBool::new(true));
            let task_active = active.clone();
            state.tool_anim = Some(active);
            let icon_s = icon.to_string();
            let display_s = display;
            let is_long = matches!(name.as_str(), "agent" | "explore");
            let spin_chars = ["·", "✢", "✳", "✶", "✻", "✽"];
            println!();
            tokio::spawn(async move {
                let mut frame = 0usize;
                while task_active.load(Ordering::Relaxed) {
                    let spin = spin_chars[frame % spin_chars.len()];
                    if is_long {
                        print!("\r  {CYAN}{icon_s}{RESET}  {DIM}{spin}  {ORANGE}{display_s}{RESET}\x1b[K");
                    } else {
                        let blink = if (frame / 4) % 2 == 0 { CYAN } else { DIM };
                        print!("\r  {blink}{icon_s}{RESET}  {DIM}{display_s}{RESET}\x1b[K");
                    }
                    io::stdout().flush().ok();
                    frame += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            });
        }

        EngineEvent::ToolInput { json_chunk } => {
            state.tool_json_buf.push_str(&json_chunk);
        }

        EngineEvent::ToolResult { name, output, is_error } => {
            use super::super::terminal::RED;

            if let Some(anim) = state.tool_anim.take() {
                anim.store(false, Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_millis(10));
                let icon = tool_icon(&state.current_tool_name);
                let tname = state.current_tool_name.clone();
                print!("\r  {CYAN}{icon}{RESET}  {DIM}{tname}{RESET}");
                io::stdout().flush().ok();
            }

            let summary = summarize_tool_input(&state.current_tool_name, &state.tool_json_buf);
            if !summary.is_empty() {
                print!("  {DIM}{summary}{RESET}");
                io::stdout().flush().ok();
            }
            let tool_json = std::mem::take(&mut state.tool_json_buf);

            if is_error {
                println!();
                println!("    {RED}✗  {}{RESET}", first_line(&output));
            } else {
                match state.current_tool_name.as_str() {
                    "file_edit" => {
                        println!();
                        super::render_diff::render_edit_diff(&tool_json);
                    }
                    "file_write" => {
                        println!();
                        super::render_diff::render_write_preview(&tool_json);
                    }
                    "read" | "glob" | "grep" | "web_fetch" | "web_search" => {
                        let lines: Vec<&str> = output.lines().collect();
                        if lines.is_empty() || output == "(no output)" {
                            println!("  {GREEN}✓{RESET}");
                        } else {
                            println!("  {DIM}({} lines){RESET}", lines.len());
                        }
                    }
                    "agent" | "explore" => {
                        println!();
                        if output.is_empty() || output == "(no output)" {
                            println!("    {GREEN}✓{RESET}");
                        } else {
                            let lines: Vec<&str> = output.lines().collect();
                            let show = lines.len().min(4);
                            for line in &lines[..show] {
                                println!("    {DIM}{line}{RESET}");
                            }
                            if lines.len() > show {
                                println!("    {DIM}… {} more lines{RESET}", lines.len() - show);
                            }
                        }
                    }
                    _ if output.is_empty() || output == "(no output)" => {
                        println!("  {GREEN}✓{RESET}");
                    }
                    _ => {
                        println!();
                        let lines: Vec<&str> = output.lines().collect();
                        let show = lines.len().min(5);
                        for line in &lines[..show] {
                            println!("    {DIM}{line}{RESET}");
                        }
                        if lines.len() > show {
                            println!("    {DIM}… {} more lines{RESET}", lines.len() - show);
                        }
                    }
                }
            }
            let _ = name;
        }

        EngineEvent::Usage { input_tokens, output_tokens } => {
            state.turn_input += input_tokens;
            state.turn_output += output_tokens;
            if let Some(ref mut s) = state.spinner { s.tokens += output_tokens; }
        }

        EngineEvent::HookOutput { source, output } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            for line in output.lines() {
                println!("  {DIM}[hook:{source}] {line}{RESET}");
            }
        }

        EngineEvent::Compacted { original_turns } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            println!();
            println!("  {DIM}{MAGENTA}◆  compacted — {original_turns} messages summarized{RESET}");
        }

        EngineEvent::ModeChanged { mode } => {
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            println!("\n  {CYAN}{BOLD}◆  {}{RESET}  {DIM}{}{RESET}", mode.label(), mode.description());
        }

        EngineEvent::TurnComplete => {
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            if state.in_thinking { print!("\r\x1b[2K"); state.in_thinking = false; state.spinner = None; }
            if state.turn_input > 0 || state.turn_output > 0 {
                let i = fmt_tokens(state.turn_input);
                let o = fmt_tokens(state.turn_output);
                println!("  {DIM}∙  {i} in  ·  {o} out{RESET}");
                state.turn_input = 0;
                state.turn_output = 0;
            }
            println!();
        }

        EngineEvent::Error(msg) => {
            if state.in_text { flush_line_buf(state); state.in_text = false; }
            render_error_box(&msg);
        }
    }
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1000 { format!("{:.1}K", n as f64 / 1000.0) } else { format!("{n}") }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}
