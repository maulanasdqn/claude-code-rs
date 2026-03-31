use std::io::{self, Write};

use claude_rust_engine::EngineEvent;

use super::terminal::{
    BOLD, CYAN, DIM, GREEN, ITALIC, MAGENTA, RED, RESET, YELLOW, summarize_tool_input, tool_icon,
};

pub struct RenderState {
    pub in_text: bool,
    pub in_thinking: bool,
    tool_json_buf: String,
    current_tool_name: String,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            in_text: false,
            in_thinking: false,
            tool_json_buf: String::new(),
            current_tool_name: String::new(),
        }
    }
}

pub fn render_event(event: EngineEvent, state: &mut RenderState) {
    match event {
        // ── Thinking delta ─────────────────────────────────────
        EngineEvent::ThinkingDelta(text) => {
            if state.in_text {
                print!("{RESET}");
                state.in_text = false;
            }
            if !state.in_thinking {
                print!("\n  {DIM}{ITALIC}💭 ");
                state.in_thinking = true;
            }
            let indented = text.replace('\n', &format!("\n     "));
            print!("{indented}");
            io::stdout().flush().ok();
        }

        // ── Streaming text ──────────────────────────────────────
        EngineEvent::TextDelta(text) => {
            if state.in_thinking {
                print!("{RESET}");
                println!();
                state.in_thinking = false;
            }
            if !state.in_text {
                print!("\n  {CYAN}⎿{RESET} {BOLD}");
                state.in_text = true;
            }
            // Indent continuation lines to align with the ⎿ prefix
            let indented = text.replace('\n', &format!("\n    "));
            print!("{indented}");
            io::stdout().flush().ok();
        }

        // ── Tool invocation start ───────────────────────────────
        EngineEvent::ToolStart { name, .. } => {
            if state.in_thinking {
                print!("{RESET}");
                println!();
                state.in_thinking = false;
            }
            if state.in_text {
                print!("{RESET}");
                println!();
                state.in_text = false;
            }
            state.tool_json_buf.clear();
            state.current_tool_name = name.clone();

            let icon = tool_icon(&name);
            print!("\n  {DIM}{icon} {YELLOW}{BOLD}{name}{RESET}{DIM} ");
            io::stdout().flush().ok();
        }

        // ── Tool input JSON delta (streaming) ───────────────────
        EngineEvent::ToolInput { json_chunk } => {
            state.tool_json_buf.push_str(&json_chunk);
            // Don't print raw JSON chunks — we'll summarize on result
        }

        // ── Tool result ─────────────────────────────────────────
        EngineEvent::ToolResult {
            name,
            output,
            is_error,
        } => {
            // Print the summarized input line
            let summary = summarize_tool_input(&state.current_tool_name, &state.tool_json_buf);
            println!("{ITALIC}{summary}{RESET}");
            state.tool_json_buf.clear();

            if is_error {
                println!("    {RED}✗ {output}{RESET}");
            } else if output.is_empty() || output == "(no output)" {
                println!("    {DIM}(no output){RESET}");
            } else {
                // Show a truncated, neatly indented preview
                let max_lines = 20;
                let max_chars = 1000;
                let preview = if output.len() > max_chars {
                    format!("{}…", &output[..max_chars])
                } else {
                    output.clone()
                };

                let lines: Vec<&str> = preview.lines().collect();
                let show = lines.len().min(max_lines);

                // Dim separator
                println!("    {DIM}┄┄┄{RESET}");
                for line in &lines[..show] {
                    println!("    {DIM}{line}{RESET}");
                }
                if lines.len() > max_lines {
                    let remaining = lines.len() - max_lines;
                    println!("    {DIM}… {remaining} more lines{RESET}");
                }
            }
            let _ = name; // already used via state.current_tool_name
        }

        // ── Usage stats ─────────────────────────────────────────
        EngineEvent::Usage { .. } => {
            // Tracked silently in main
        }

        // ── Context compacted ───────────────────────────────────
        EngineEvent::Compacted { original_turns } => {
            if state.in_text {
                print!("{RESET}");
                state.in_text = false;
            }
            println!();
            println!(
                "  {DIM}{MAGENTA}⟳  Context compacted — {original_turns} messages summarized{RESET}"
            );
        }

        // ── Plan mode changed ──────────────────────────────────
        EngineEvent::PlanModeChanged { enabled } => {
            if state.in_text {
                print!("{RESET}");
                state.in_text = false;
            }
            if enabled {
                println!(
                    "\n  {MAGENTA}{BOLD}📋 Plan mode activated{RESET} {DIM}— only read-only tools available{RESET}"
                );
            } else {
                println!(
                    "\n  {GREEN}{BOLD}✓ Plan mode deactivated{RESET} {DIM}— all tools available{RESET}"
                );
            }
        }

        // ── Turn complete ───────────────────────────────────────
        EngineEvent::TurnComplete => {
            if state.in_text {
                print!("{RESET}");
                state.in_text = false;
            }
            if state.in_thinking {
                print!("{RESET}");
                state.in_thinking = false;
            }
            println!();
        }

        // ── Error ───────────────────────────────────────────────
        EngineEvent::Error(msg) => {
            if state.in_text {
                print!("{RESET}");
                state.in_text = false;
            }
            eprintln!("\n  {RED}{BOLD}✗ error:{RESET}{RED} {msg}{RESET}\n");
        }
    }
}

// ── Formatted cost display ──────────────────────────────────────────

pub fn render_cost(input_tokens: u64, output_tokens: u64) {
    let input_cost = input_tokens as f64 * 3.0 / 1_000_000.0;
    let output_cost = output_tokens as f64 * 15.0 / 1_000_000.0;
    let total_cost = input_cost + output_cost;

    let total_tokens = input_tokens + output_tokens;

    println!();
    println!("  {DIM}┌─────────────────────────────────────┐{RESET}");
    println!("  {DIM}│{RESET} {BOLD}Token Usage{RESET}                        {DIM}│{RESET}");
    println!("  {DIM}├─────────────────────────────────────┤{RESET}");
    println!(
        "  {DIM}│{RESET}   Input    {CYAN}{input_tokens:>10}{RESET} {DIM}(${input_cost:.4}){RESET}  {DIM}│{RESET}"
    );
    println!(
        "  {DIM}│{RESET}   Output   {CYAN}{output_tokens:>10}{RESET} {DIM}(${output_cost:.4}){RESET}  {DIM}│{RESET}"
    );
    println!("  {DIM}├─────────────────────────────────────┤{RESET}");
    println!(
        "  {DIM}│{RESET}   Total    {BOLD}{total_tokens:>10}{RESET}  {GREEN}${total_cost:.4}{RESET}   {DIM}│{RESET}"
    );
    println!("  {DIM}└─────────────────────────────────────┘{RESET}");
    println!();
}
