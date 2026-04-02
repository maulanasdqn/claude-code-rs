use std::io::{self, Write};

use claude_rust_types::EngineEvent;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";

pub fn print_banner(cwd: &str, model: &str) {
    let short_model = model.trim_start_matches("claude-");
    eprintln!();
    eprintln!("  {BOLD}{CYAN}◆{RESET}  {BOLD}Rusty Claude{RESET}  {DIM}v{}{RESET}", env!("CARGO_PKG_VERSION"));
    eprintln!("  {DIM}cwd{RESET}   {cwd}");
    eprintln!("  {DIM}model{RESET} {short_model}");
    eprintln!();
}

pub fn render_event(event: &EngineEvent, json_mode: bool) {
    match event {
        EngineEvent::TextDelta(text) => {
            if json_mode {
                return;
            }
            print!("{text}");
            io::stdout().flush().ok();
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
                let icon = if *is_error { format!("{RED}✗") } else { format!("{GREEN}✓") };
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

pub fn render_error(msg: &str) {
    eprintln!("  {RED}{BOLD}error:{RESET} {msg}");
}

pub fn print_prompt() {
    eprint!("{BOLD}{CYAN}❯{RESET} ");
    io::stderr().flush().ok();
}
