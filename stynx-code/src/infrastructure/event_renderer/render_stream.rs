use std::io::{self, Write};

use stynx_code_types::EngineEvent;

use super::super::terminal::{BOLD, DIM, GREEN, RED, RESET, YELLOW};

pub fn render_stream_event(event: &EngineEvent, json_mode: bool) {
    match event {
        EngineEvent::TextDelta(text) => {
            if !json_mode {
                print!("{text}");
                io::stdout().flush().ok();
            }
        }
        EngineEvent::ThinkingDelta(_) => {}
        EngineEvent::ToolStart { name, .. } => {
            if !json_mode {
                eprint!("\n  {DIM}{YELLOW}⚡ {name}{RESET} ");
                io::stderr().flush().ok();
            }
        }
        EngineEvent::ToolInput { .. } => {}
        EngineEvent::ToolResult { name, is_error, .. } => {
            if !json_mode {
                let icon = if *is_error {
                    format!("{RED}✗")
                } else {
                    format!("{GREEN}✓")
                };
                eprintln!("{icon}{RESET} {DIM}{name}{RESET}");
            }
        }
        EngineEvent::Usage { input_tokens, output_tokens } => {
            if !json_mode && (*input_tokens > 0 || *output_tokens > 0) {
                eprintln!(
                    "  {DIM}tokens: {input_tokens} in / {output_tokens} out{RESET}"
                );
            }
        }
        EngineEvent::TurnComplete => {
            if !json_mode {
                println!();
            }
        }
        EngineEvent::Error(msg) => {
            eprintln!("  {RED}{BOLD}error:{RESET} {msg}");
        }
        EngineEvent::Compacted { original_turns } => {
            if !json_mode {
                eprintln!("  {DIM}(compacted {original_turns} turns){RESET}");
            }
        }
        EngineEvent::ModeChanged { mode } => {
            if !json_mode {
                eprintln!("  {DIM}mode → {}{RESET}", mode.label());
            }
        }
        EngineEvent::HookOutput { source, output } => {
            if !json_mode {
                eprintln!("  {DIM}[{source}] {output}{RESET}");
            }
        }
    }
}
