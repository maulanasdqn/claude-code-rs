mod infrastructure;

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use claude_rust_commands::{CommandResult, execute_command, expand_file_references, parse_command};
use claude_rust_engine::QueryEngine;
use claude_rust_memory::FileSessionRepository;
use claude_rust_permission::InteractivePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{BashTool, ReadTool, ToolRegistry};
use claude_rust_types::{Conversation, Message};

use infrastructure::event_renderer::render_event;
use infrastructure::terminal::{
    DIM, RED, RESET, make_system_prompt, print_banner, prompt_resume, read_user_input,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(io::stderr)
        .init();

    let credential = match claude_rust_auth::resolve_credential() {
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

    let permission = Arc::new(InteractivePermissionChecker);
    let engine = Arc::new(QueryEngine::new(provider.clone(), registry, permission));

    let session_repo: Arc<dyn claude_rust_memory::SessionRepository> = match FileSessionRepository::new() {
        Ok(repo) => Arc::new(repo),
        Err(e) => {
            tracing::warn!("failed to init session repository: {e}");
            Arc::new(FileSessionRepository::with_dir(
                std::path::PathBuf::from(".claude-code-rs/sessions"),
            ))
        }
    };

    print_banner();

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".into());

    let system_prompt = make_system_prompt(&cwd);

    let mut conversation = match claude_rust_memory::load_session(&session_repo).await {
        Ok(Some(prev)) if !prev.messages.is_empty() => {
            if prompt_resume() {
                let mut c = prev;
                if c.system.is_none() {
                    c.system = Some(system_prompt.clone());
                }
                println!("{DIM}Session resumed ({} messages).{RESET}\n", c.messages.len());
                c
            } else {
                let mut c = Conversation::default();
                c.system = Some(system_prompt.clone());
                c
            }
        }
        _ => {
            let mut c = Conversation::default();
            c.system = Some(system_prompt.clone());
            c
        }
    };

    loop {
        let input = match read_user_input() {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        if let Some(cmd) = parse_command(&input) {
            match execute_command(cmd) {
                CommandResult::Output(text) => {
                    println!("\n{text}\n");
                    continue;
                }
                CommandResult::ReplaceConversation(mut c) => {
                    c.system = Some(system_prompt.clone());
                    conversation = c;
                    println!("\n{DIM}Conversation cleared.{RESET}\n");
                    continue;
                }
                CommandResult::Quit => break,
            }
        }

        let expanded = expand_file_references(&input);
        conversation.push(Message::user(&expanded));

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
                render_event(event, &mut in_text);
            })
            .await;

        spinning.store(false, Ordering::Relaxed);
        let _ = spinner_handle.await;

        match result {
            Ok(updated) => {
                conversation = updated;
                if let Err(e) = claude_rust_memory::save_session(&session_repo, &conversation).await {
                    tracing::warn!("failed to save session: {e}");
                }
            }
            Err(e) => {
                eprintln!("{RED}error: {e}{RESET}\n");
            }
        }
    }

    if let Err(e) = claude_rust_memory::save_session(&session_repo, &conversation).await {
        tracing::warn!("failed to save final session: {e}");
    }

    println!("{DIM}Goodbye!{RESET}");
}
