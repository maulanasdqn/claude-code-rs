use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};

use claude_rust_commands::expand_message_content;
use claude_rust_engine::QueryEngine;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};
use tokio::sync::mpsc;

use super::app_actions::{copy_last_response, do_undo, expand_with_pins, handle_add, handle_init, handle_logout, run_shell, save_session, show_files, show_skills, toggle_reflect};
use super::app_display::{show_todos, show_transcript};
use super::app_help::print_help;
use super::command_handler::{handle_slash_command};
use super::command_types::CommandAction;
use super::conductor::CONDUCTOR_SYSTEM;
use super::event_renderer::{render_cost, render_error_box, render_exit_summary};
use super::input_history::{append_history, load_history};
use super::run_engine::run_engine;
use super::self_correct::run_self_correct;
use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET, clear_pasted_images, print_banner, read_user_input, set_current_model, take_pasted_images};

#[allow(clippy::too_many_arguments)]
pub async fn run_loop(
    engine: Arc<QueryEngine>,
    conductor_engine: Arc<QueryEngine>,
    reflect_engine: Arc<QueryEngine>,
    session_repo: Arc<dyn claude_rust_memory::SessionRepository>,
    provider: Arc<AnthropicProvider>,
    config: claude_rust_config::Settings,
    mode_flag: Arc<AtomicU8>,
    cwd: String,
    system_prompt: String,
    mut conversation: Conversation,
    skills: Vec<Skill>,
    pause_flag: Arc<AtomicBool>,
    permission: Arc<ConfigAwarePermissionChecker>,
) {
    let session_start = std::time::Instant::now();
    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let reflect_enabled = Arc::new(AtomicBool::new(false));
    let undo_stack = engine.undo_stack();
    let mut prompt_history: Vec<String> = load_history(&cwd);
    let mut pinned_files: Vec<String> = Vec::new();
    let model_id = provider.model_name();
    set_current_model(&model_id);
    let skill_names: Vec<(String, String)> = skills.iter()
        .filter(|s| s.user_invocable).map(|s| (s.name.clone(), s.description.clone())).collect();
    let (bg_tx, mut bg_rx) = mpsc::unbounded_channel::<Result<Conversation, claude_rust_errors::AppError>>();
    let mut bg_count: usize = 0;

    loop {
        let input = match read_user_input(&mode_flag, &prompt_history, &skill_names, !conversation.messages.is_empty()) {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        while let Ok(result) = bg_rx.try_recv() {
            match result {
                Ok(updated) => { conversation = updated; save_session(&session_repo, &conversation).await; println!("  {CYAN}{BOLD}⌁{RESET}  {DIM}Background task completed{RESET}\n"); }
                Err(e) if e.is_interrupted() => { println!("  {DIM}⌁ Background task interrupted{RESET}\n"); }
                Err(e) => render_error_box(&e.to_string()),
            }
        }

        if let Some(signal) = input.strip_prefix('\x00') {
            clear_pasted_images();
            if let Some(bg_text) = signal.strip_prefix("background:") {
                let expanded = expand_with_pins(bg_text, &pinned_files);
                let mut bg_conv = conversation.clone();
                bg_conv.push(Message { role: Role::User, content: expand_message_content(&expanded) });
                bg_count += 1;
                let task_num = bg_count;
                println!("  {DIM}⌁ Task #{task_num} sent to background{RESET}\n");
                let (bg_engine, bg_ti, bg_to, bg_mf, bg_mid, bg_cwd, bg_pf, bg_tx) =
                    (engine.clone(), total_input.clone(), total_output.clone(), mode_flag.clone(), model_id.clone(), cwd.clone(), pause_flag.clone(), bg_tx.clone());
                tokio::spawn(async move {
                    let result = run_engine(&bg_engine, bg_conv, &bg_ti, &bg_to, &bg_mf, &bg_mid, &bg_cwd, &bg_pf).await;
                    let _ = bg_tx.send(result);
                });
                prompt_history.push(bg_text.to_string());
                append_history(bg_text, &cwd);
                continue;
            }
            match signal {
                "transcript" => { show_transcript(&conversation); continue; }
                "todos" => { show_todos(&cwd); continue; }
                _ => continue,
            }
        }

        let trimmed = input.trim();

        if trimmed == "/help" { print_help(&skills); continue; }
        if let Some(task) = trimmed.strip_prefix("/conductor").map(str::trim).filter(|s| !s.is_empty()) {
            handle_conductor_cmd(task, &conductor_engine, &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await;
            continue;
        }
        if trimmed == "/cost" { render_cost(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed)); continue; }
        if let Some(shell_cmd) = trimmed.strip_prefix('!') { run_shell(shell_cmd); continue; }
        if trimmed == "/init" { handle_init(&engine, &mut conversation, &session_repo, &total_input, &total_output, &mode_flag, &cwd, &model_id, &pause_flag).await; continue; }
        if let Some(rest) = trimmed.strip_prefix("/undo") { print!("{}", do_undo(&undo_stack, rest.trim().parse().unwrap_or(1)).await); continue; }
        if let Some(path) = trimmed.strip_prefix("/add ") { handle_add(path, &mut pinned_files); continue; }
        if trimmed == "/skills" { show_skills(&skills); continue; }
        if trimmed == "/files" { show_files(&pinned_files); continue; }
        if trimmed == "/copy" { copy_last_response(&conversation); continue; }
        if trimmed == "/login" { println!("  {DIM}Run {BOLD}claude login{RESET}{DIM} in your terminal to authenticate.{RESET}\n"); continue; }
        if trimmed == "/logout" { handle_logout(); continue; }
        if trimmed == "/vim" { println!("  {DIM}Vim mode is always active. Use Esc to enter normal mode.{RESET}\n"); continue; }
        if trimmed == "/reflect" {
            let was = toggle_reflect(&reflect_enabled);
            if was { println!("  {DIM}◆ Self-correction disabled{RESET}\n"); }
            else { println!("  {CYAN}{BOLD}◆ Self-correction enabled{RESET}  {DIM}— responses will be auto-reviewed against system standards{RESET}\n"); }
            continue;
        }
        if trimmed == "/version" { println!("  {DIM}claude-rust v{}{RESET}\n", env!("CARGO_PKG_VERSION")); continue; }

        if input.starts_with('/') {
            match handle_slash_command(&input, &provider, &config, &mode_flag, &system_prompt, &cwd, &conversation, &skills).await {
                Some(CommandAction::Output(text)) => {
                    print!("{text}");
                    set_current_model(&provider.model_name());
                    continue;
                }
                Some(CommandAction::ReplaceConversation(c)) => {
                    let msg_count = c.messages.len();
                    conversation = c;
                    let current_model = provider.model_name();
                    set_current_model(&current_model);
                    print!("\x1b[2J\x1b[H");
                    print_banner(&cwd, &current_model);
                    println!("  {DIM}✓ {msg_count} messages kept.{RESET}\n");
                    continue;
                }
                Some(CommandAction::SendToEngine(msg, allowed_tools)) => {
                    if !allowed_tools.is_empty() { permission.set_skill_allow_rules(allowed_tools); }
                    conversation.push(Message { role: Role::User, content: expand_message_content(&msg) });
                    match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
                        Ok(updated) => { conversation = updated; save_session(&session_repo, &conversation).await; }
                        Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
                        Err(e) => render_error_box(&e.to_string()),
                    }
                    permission.clear_skill_allow_rules();
                    continue;
                }
                Some(CommandAction::Quit) => break,
                Some(CommandAction::Continue) => continue,
                None => { let cmd = trimmed.split_whitespace().next().unwrap_or(trimmed); println!("  {DIM}Unknown command: {cmd}  (try /help or /skills){RESET}\n"); continue; }
            }
        }

        prompt_history.push(input.clone());
        append_history(&input, &cwd);
        let expanded = expand_with_pins(&input, &pinned_files);
        let mut content = expand_message_content(&expanded);
        let images = take_pasted_images();
        if !images.is_empty() {
            let n = images.len();
            for (media_type, data) in images { content.push(claude_rust_types::ContentBlock::Image { media_type, data }); }
            let s = if n == 1 { "" } else { "s" };
            println!("  {DIM}📎 {n} image{s} attached{RESET}");
        }
        conversation.push(Message { role: Role::User, content });
        match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
            Ok(mut updated) => {
                if reflect_enabled.load(Ordering::Relaxed) { run_self_correct(&mut updated, &reflect_engine, &system_prompt).await; }
                conversation = updated;
                save_session(&session_repo, &conversation).await;
            }
            Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
            Err(e) => render_error_box(&e.to_string()),
        }
    }

    save_session(&session_repo, &conversation).await;
    render_exit_summary(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed), session_start.elapsed());
}

async fn handle_conductor_cmd(
    task: &str,
    engine: &Arc<QueryEngine>,
    total_input: &Arc<AtomicU64>,
    total_output: &Arc<AtomicU64>,
    mode_flag: &Arc<AtomicU8>,
    model_id: &str,
    cwd: &str,
    pause_flag: &Arc<AtomicBool>,
) {
    let mut cond_conv = Conversation { system: Some(CONDUCTOR_SYSTEM.to_string()), ..Default::default() };
    cond_conv.push(Message { role: Role::User, content: expand_message_content(task) });
    match run_engine(engine, cond_conv, total_input, total_output, mode_flag, model_id, cwd, pause_flag).await {
        Ok(_) => {}
        Err(e) if e.is_interrupted() => {}
        Err(e) => render_error_box(&e.to_string()),
    }
}
