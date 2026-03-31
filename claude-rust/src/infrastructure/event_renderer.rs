use std::io::{self, Write};

use claude_rust_engine::EngineEvent;

use super::terminal::{BOLD, DIM, RED, RESET, YELLOW};

pub fn render_event(event: EngineEvent, in_text: &mut bool) {
    match event {
        EngineEvent::TextDelta(text) => {
            if !*in_text {
                print!("\n{BOLD}");
                *in_text = true;
            }
            print!("{text}");
            io::stdout().flush().ok();
        }
        EngineEvent::ToolStart { name, .. } => {
            if *in_text {
                println!("{RESET}");
                *in_text = false;
            }
            print!("\n  {DIM}{YELLOW}[tool: {name}]{RESET} ");
            io::stdout().flush().ok();
        }
        EngineEvent::ToolInput { json_chunk } => {
            print!("{DIM}{json_chunk}{RESET}");
            io::stdout().flush().ok();
        }
        EngineEvent::ToolResult {
            name,
            output,
            is_error,
        } => {
            println!();
            if is_error {
                println!("  {RED}[{name} error]: {output}{RESET}");
            } else {
                let preview = if output.len() > 500 {
                    format!("{}...", &output[..500])
                } else {
                    output
                };
                for line in preview.lines().take(15) {
                    println!("  {DIM}| {line}{RESET}");
                }
                if preview.lines().count() > 15 {
                    println!("  {DIM}| ...{RESET}");
                }
            }
        }
        EngineEvent::TurnComplete => {
            if *in_text {
                print!("{RESET}");
                *in_text = false;
            }
            println!("\n");
        }
        EngineEvent::Error(msg) => {
            if *in_text {
                print!("{RESET}");
                *in_text = false;
            }
            eprintln!("\n{RED}error: {msg}{RESET}\n");
        }
    }
}
