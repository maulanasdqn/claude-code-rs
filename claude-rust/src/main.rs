mod infrastructure;

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use claude_rust_commands::{CommandResult, execute_command, expand_file_references, parse_command};
use claude_rust_config::load_config;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_memory::FileSessionRepository;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_tools::{
    AskUserTool, BashTool, EnterPlanModeTool, ExitPlanModeTool, FileEditTool, FileWriteTool,
    GlobTool, GrepTool, ReadTool, ToolRegistry, WebFetchTool, WebSearchTool,
};
use claude_rust_types::{Conversation, Message};

use infrastructure::event_renderer::{RenderState, render_cost, render_event};
use infrastructure::terminal::{
    BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET, SPINNER_VERBS,
    make_system_prompt, print_banner, prompt_resume, read_user_input,
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

    // Load config
    let config = load_config();

    let credential = match claude_rust_auth::resolve_credential() {
        Ok(cred) => cred,
        Err(e) => {
            eprintln!("  {RED}{BOLD}✗{RESET}{RED} {e}{RESET}");
            std::process::exit(1);
        }
    };

    let provider = Arc::new(AnthropicProvider::new(credential));

    // Apply model from config if set
    if let Some(ref model) = config.model {
        provider.set_model(model);
    }

    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(BashTool));
    registry.register(Arc::new(ReadTool));
    registry.register(Arc::new(FileWriteTool));
    registry.register(Arc::new(FileEditTool));
    registry.register(Arc::new(GlobTool));
    registry.register(Arc::new(GrepTool));
    registry.register(Arc::new(AskUserTool));
    registry.register(Arc::new(WebFetchTool));
    registry.register(Arc::new(WebSearchTool));
    registry.register(Arc::new(EnterPlanModeTool));
    registry.register(Arc::new(ExitPlanModeTool));
    let registry = Arc::new(registry);

    let permission = Arc::new(ConfigAwarePermissionChecker::new(
        config.permissions.clone(),
    ));
    let engine = Arc::new(QueryEngine::new(provider.clone(), registry, permission));
    let plan_mode = engine.plan_mode_flag();

    let session_repo: Arc<dyn claude_rust_memory::SessionRepository> =
        match FileSessionRepository::new() {
            Ok(repo) => Arc::new(repo),
            Err(e) => {
                tracing::warn!("failed to init session repository: {e}");
                Arc::new(FileSessionRepository::with_dir(
                    std::path::PathBuf::from(".claude-code-rs/sessions"),
                ))
            }
        };

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".into());

    print_banner(&cwd);

    let system_prompt = make_system_prompt(&cwd);

    let mut conversation = match claude_rust_memory::load_session(&session_repo).await {
        Ok(Some(prev)) if !prev.messages.is_empty() => {
            if prompt_resume() {
                let mut c = prev;
                if c.system.is_none() {
                    c.system = Some(system_prompt.clone());
                }
                println!(
                    "  {DIM}↻ Session resumed ({} messages){RESET}\n",
                    c.messages.len()
                );
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

    // Cumulative token tracking
    let total_input_tokens = Arc::new(AtomicU64::new(0));
    let total_output_tokens = Arc::new(AtomicU64::new(0));

    loop {
        let input = match read_user_input() {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        // ── /cost ──────────────────────────────────────────────
        if input.trim() == "/cost" {
            render_cost(
                total_input_tokens.load(Ordering::Relaxed),
                total_output_tokens.load(Ordering::Relaxed),
            );
            continue;
        }

        // ── Slash commands ───────────────────────────────────────
        if let Some(cmd) = parse_command(&input) {
            // Handle /model specially (needs provider access)
            if let claude_rust_commands::SlashCommand::Model(ref name) = cmd {
                if name.is_empty() {
                    println!(
                        "\n  {DIM}Current model:{RESET} {BOLD}{CYAN}{}{RESET}\n",
                        provider.model_name()
                    );
                } else {
                    provider.set_model(name);
                    println!("\n  {DIM}Model →{RESET} {BOLD}{CYAN}{name}{RESET}\n");
                }
                continue;
            }

            // Handle /config (needs config data)
            if matches!(cmd, claude_rust_commands::SlashCommand::Config) {
                let json = serde_json::to_string_pretty(&config).unwrap_or_default();
                let result = claude_rust_commands::infrastructure::handlers::handle_config(&json);
                if let CommandResult::Output(text) = result {
                    println!("\n{text}\n");
                }
                continue;
            }

            // Handle /permissions (needs config data)
            if matches!(cmd, claude_rust_commands::SlashCommand::Permissions) {
                let result = claude_rust_commands::infrastructure::handlers::handle_permissions(
                    &config.permissions.allow,
                    &config.permissions.deny,
                );
                if let CommandResult::Output(text) = result {
                    println!("\n{text}\n");
                }
                continue;
            }

            // Handle /plan (needs engine state)
            if matches!(cmd, claude_rust_commands::SlashCommand::Plan) {
                let current = plan_mode.load(Ordering::Relaxed);
                let new_state = !current;
                plan_mode.store(new_state, Ordering::Relaxed);
                if new_state {
                    println!("\n  {MAGENTA}{BOLD}📋 Plan mode activated{RESET} {DIM}— only read-only tools available{RESET}\n");
                } else {
                    println!("\n  {GREEN}{BOLD}✓ Plan mode deactivated{RESET} {DIM}— all tools available{RESET}\n");
                }
                continue;
            }

            match execute_command(cmd).await {
                CommandResult::Output(text) => {
                    println!("\n{text}\n");
                    continue;
                }
                CommandResult::ReplaceConversation(mut c) => {
                    c.system = Some(system_prompt.clone());
                    conversation = c;
                    println!("\n  {DIM}✓ Conversation cleared.{RESET}\n");
                    continue;
                }
                CommandResult::Quit => break,
            }
        }

        // ── Normal message ─────────────────────────────────────
        let expanded = expand_file_references(&input);
        conversation.push(Message::user(&expanded));

        // Spinner with rotating verbs
        let spinning = Arc::new(AtomicBool::new(true));
        let spinning_clone = spinning.clone();
        let spinner_handle = tokio::spawn(async move {
            const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut frame = 0;
            let mut verb_idx = 0;
            let mut ticks = 0u64;
            while spinning_clone.load(Ordering::Relaxed) {
                let verb = SPINNER_VERBS[verb_idx % SPINNER_VERBS.len()];
                let dots = match (ticks / 4) % 4 {
                    0 => "",
                    1 => ".",
                    2 => "..",
                    _ => "...",
                };
                eprint!(
                    "\r  {CYAN}{}{RESET} {DIM}{verb}{dots}{RESET}\x1b[K",
                    FRAMES[frame % FRAMES.len()]
                );
                io::stderr().flush().ok();
                frame += 1;
                ticks += 1;
                // Rotate verb every ~2 seconds (25 ticks at 80ms)
                if ticks % 25 == 0 {
                    verb_idx += 1;
                }
                tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            }
            eprint!("\r\x1b[2K");
            io::stderr().flush().ok();
        });

        let mut render_state = RenderState::new();
        let spinning_ref = spinning.clone();
        let input_counter = total_input_tokens.clone();
        let output_counter = total_output_tokens.clone();

        let result = engine
            .run(conversation.clone(), move |event| {
                if spinning_ref.load(Ordering::Relaxed) {
                    spinning_ref.store(false, Ordering::Relaxed);
                    // Small delay to let spinner clear line
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                // Track usage tokens
                if let EngineEvent::Usage {
                    input_tokens,
                    output_tokens,
                } = &event
                {
                    if *input_tokens > 0 {
                        input_counter.fetch_add(*input_tokens, Ordering::Relaxed);
                    }
                    if *output_tokens > 0 {
                        output_counter.fetch_add(*output_tokens, Ordering::Relaxed);
                    }
                }
                render_event(event, &mut render_state);
            })
            .await;

        spinning.store(false, Ordering::Relaxed);
        let _ = spinner_handle.await;

        match result {
            Ok(updated) => {
                conversation = updated;
                if let Err(e) =
                    claude_rust_memory::save_session(&session_repo, &conversation).await
                {
                    tracing::warn!("failed to save session: {e}");
                }
            }
            Err(e) => {
                eprintln!("  {RED}{BOLD}✗{RESET}{RED} {e}{RESET}\n");
            }
        }
    }

    if let Err(e) = claude_rust_memory::save_session(&session_repo, &conversation).await {
        tracing::warn!("failed to save final session: {e}");
    }

    println!("\n  {DIM}Goodbye!{RESET}\n");
}
