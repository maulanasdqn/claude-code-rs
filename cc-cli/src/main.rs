use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cc_engine::{EngineEvent, QueryEngine};
use cc_provider::AnthropicProvider;
use cc_tools::{BashTool, ReadTool, ToolRegistry};
use cc_types::{AllowAll, Conversation, Message};

const BLUE: &str = "\x1b[34m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";

fn print_banner() {
    println!("{BOLD}{BLUE}╭─────────────────────────────────╮{RESET}");
    println!("{BOLD}{BLUE}│  Claude Code (Rust)  v0.1.0     │{RESET}");
    println!("{BOLD}{BLUE}╰─────────────────────────────────╯{RESET}");
    println!("{DIM}Type a message to chat. Ctrl+C to exit.{RESET}");
    println!();
}

fn read_user_input() -> Option<String> {
    print!("{BOLD}{GREEN}> {RESET}");
    io::stdout().flush().ok()?;

    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                Some(String::new())
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(io::stderr)
        .init();

    let credential = match cc_auth::resolve_credential() {
        Ok(cred) => cred,
        Err(e) => {
            eprintln!("{RED}error:{RESET} {e}");
            std::process::exit(1);
        }
    };

    let provider = Arc::new(AnthropicProvider::new(credential));

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(AllowAll);
    let engine = Arc::new(QueryEngine::new(provider, registry, permission));

    print_banner();

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".into());

    let mut conversation = Conversation::default();
    conversation.system = Some(format!(
        "You are Claude Code, an interactive CLI assistant. \
         The user's working directory is: {cwd}. \
         You have access to `bash` and `read` tools. \
         Be concise and helpful. Use tools when needed to answer questions."
    ));

    loop {
        let input = match read_user_input() {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        if input == "/quit" || input == "/exit" {
            break;
        }

        conversation.push(Message::user(&input));

        let spinning = Arc::new(AtomicBool::new(true));
        let spinning_clone = spinning.clone();
        let spinner_handle = tokio::spawn(async move {
            const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut i = 0;
            while spinning_clone.load(Ordering::Relaxed) {
                eprint!("\r  {DIM}{}{RESET} thinking...", FRAMES[i % FRAMES.len()]);
                io::stderr().flush().ok();
                i += 1;
                tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            }
            eprint!("\r\x1b[2K");
            io::stderr().flush().ok();
        });

        let mut in_text = false;
        let spinning_ref = spinning.clone();

        let result = engine
            .run(conversation.clone(), move |event| {
                if spinning_ref.load(Ordering::Relaxed) {
                    spinning_ref.store(false, Ordering::Relaxed);
                }

                match event {
                    EngineEvent::TextDelta(text) => {
                        if !in_text {
                            print!("\n{BOLD}");
                            in_text = true;
                        }
                        print!("{text}");
                        io::stdout().flush().ok();
                    }
                    EngineEvent::ToolStart { name, .. } => {
                        if in_text {
                            println!("{RESET}");
                            in_text = false;
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
                        if in_text {
                            print!("{RESET}");
                            in_text = false;
                        }
                        println!("\n");
                    }
                    EngineEvent::Error(msg) => {
                        if in_text {
                            print!("{RESET}");
                            in_text = false;
                        }
                        eprintln!("\n{RED}error: {msg}{RESET}\n");
                    }
                }
            })
            .await;

        spinning.store(false, Ordering::Relaxed);
        let _ = spinner_handle.await;

        match result {
            Ok(updated) => {
                conversation = updated;
            }
            Err(e) => {
                eprintln!("{RED}error: {e}{RESET}\n");
            }
        }
    }

    println!("{DIM}Goodbye!{RESET}");
}
