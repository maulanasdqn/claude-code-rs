use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use claude_rust_commands::expand_message_content;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};

use super::command_handler::{CommandAction, handle_slash_command};
use super::event_renderer::{RenderState, render_cost, render_error_box, render_event};
use super::spinner::spawn_spinner;
use super::terminal::{DIM, RESET, make_system_prompt, pin_working_footer, print_banner, read_user_input, unpin_working_footer};

pub async fn run_loop(
    engine: Arc<QueryEngine>,
    session_repo: Arc<dyn claude_rust_memory::SessionRepository>,
    provider: Arc<AnthropicProvider>,
    config: claude_rust_config::Settings,
    mode_flag: Arc<AtomicU8>,
    cwd: String,
    mut conversation: Conversation,
) {
    let system_prompt = make_system_prompt(&cwd);

    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let mut prompt_history: Vec<String> = Vec::new();

    loop {
        let input = match read_user_input(&mode_flag, &prompt_history) {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        if input.trim() == "/cost" {
            render_cost(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
            continue;
        }

        if input.starts_with('/') {
            match handle_slash_command(&input, &provider, &config, &mode_flag, &system_prompt, &cwd).await {
                Some(CommandAction::Output(text)) => { print!("{text}"); continue; }
                Some(CommandAction::ReplaceConversation(c)) => {
                    conversation = c;
                    print!("\x1b[2J\x1b[H");
                    print_banner(&cwd);
                    println!("  {DIM}✓ Conversation cleared.{RESET}\n");
                    continue;
                }
                Some(CommandAction::Quit) => break,
                Some(CommandAction::Continue) => continue,
                None => {}
            }
        }

        prompt_history.push(input.clone());
        let content_blocks = expand_message_content(&input);
        conversation.push(Message { role: Role::User, content: content_blocks });

        pin_working_footer();
        let (spinning, spinner_handle) = spawn_spinner(mode_flag.clone());

        let mut render_state = RenderState::new();
        let spinning_ref = spinning.clone();
        let in_counter = total_input.clone();
        let out_counter = total_output.clone();

        let result = engine
            .run(conversation.clone(), move |event| {
                if spinning_ref.load(Ordering::Relaxed) {
                    spinning_ref.store(false, Ordering::Relaxed);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                if let EngineEvent::Usage { input_tokens, output_tokens } = &event {
                    if *input_tokens > 0 { in_counter.fetch_add(*input_tokens, Ordering::Relaxed); }
                    if *output_tokens > 0 { out_counter.fetch_add(*output_tokens, Ordering::Relaxed); }
                }
                render_event(event, &mut render_state);
            })
            .await;

        spinning.store(false, Ordering::Relaxed);
        let _ = spinner_handle.await;
        unpin_working_footer();

        match result {
            Ok(updated) => {
                conversation = updated;
                if let Err(e) = claude_rust_memory::save_session(&session_repo, &conversation).await {
                    tracing::warn!("failed to save session: {e}");
                }
            }
            Err(e) => render_error_box(&e.to_string()),
        }
    }

    if let Err(e) = claude_rust_memory::save_session(&session_repo, &conversation).await {
        tracing::warn!("failed to save final session: {e}");
    }

    println!("\n  {DIM}Goodbye!{RESET}\n");
}
