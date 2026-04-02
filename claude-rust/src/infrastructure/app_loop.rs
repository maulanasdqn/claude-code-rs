use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};
use std::time::Duration;

use claude_rust_commands::expand_message_content;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::AppError;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};
use claude_rust_tui::{EventHandler, TuiApp, UiAction};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::app_actions::{expand_with_pins, save_session};
use super::command_handler::handle_slash_command;
use super::command_types::CommandAction;
use super::run_engine::run_engine_tui;
use super::skills::Skill;
use super::terminal::set_current_model;

type EngineTask = JoinHandle<Result<Conversation, AppError>>;

#[allow(clippy::too_many_arguments)]
pub async fn run_loop(
    engine: Arc<QueryEngine>,
    _conductor_engine: Arc<QueryEngine>,
    _reflect_engine: Arc<QueryEngine>,
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
    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let mut pinned_files: Vec<String> = Vec::new();
    let model_id = provider.model_name();

    let mut tui = match TuiApp::new() {
        Ok(t) => t,
        Err(e) => { eprintln!("TUI init failed: {e}"); return; }
    };
    tui.state.model_name = model_id.clone();
    tui.state.git_branch = super::terminal::git_branch();
    set_current_model(&model_id);

    for msg in &conversation.messages {
        let role = match msg.role { Role::User => "user", Role::Assistant => "assistant" };
        let text = msg.content.iter().filter_map(|b| {
            if let claude_rust_types::ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
        }).collect::<Vec<_>>().join("");
        if !text.is_empty() { tui.state.conversation.messages.push(claude_rust_tui::DisplayMessage {
            role: role.to_string(), content: text,
            tool_uses: Vec::new(), is_streaming: false,
        }); }
    }

    let (key_tx, mut key_rx) = mpsc::unbounded_channel::<crossterm::event::Event>();
    let stop_keys = Arc::new(AtomicBool::new(false));
    let stop_keys2 = stop_keys.clone();
    tokio::task::spawn_blocking(move || {
        while !stop_keys2.load(Ordering::Relaxed) {
            if crossterm::event::poll(Duration::from_millis(30)).unwrap_or(false) {
                if let Ok(ev) = crossterm::event::read() { if key_tx.send(ev).is_err() { break; } }
            }
        }
    });

    let mut engine_task: Option<(EngineTask, mpsc::UnboundedReceiver<EngineEvent>, Conversation)> = None;

    loop {
        tui.sync_pause(&pause_flag);

        if let Some((task, ev_rx, _)) = &mut engine_task {
            while let Ok(ev) = ev_rx.try_recv() { tui.state.apply_engine_event(ev); }
            if task.is_finished() {
                let (task, _, pre) = engine_task.take().unwrap();
                match task.await {
                    Ok(Ok(updated)) => { conversation = updated; save_session(&session_repo, &conversation).await; }
                    Ok(Err(e)) if e.is_interrupted() => {
                        conversation = pre;
                        if let Some(last) = tui.state.conversation.messages.last_mut() {
                            last.is_streaming = false;
                        }
                    }
                    Ok(Err(e)) => { tui.state.push_system_message(format!("Error: {e}")); conversation = pre; }
                    Err(_) => { conversation = pre; }
                }
                tui.state.is_streaming = false;
            }
        }

        tui.tick_spinner();
        if tui.is_in_alt() { tui.draw().ok(); }

        while let Ok(ev) = key_rx.try_recv() {
            match EventHandler::handle(ev, &mut tui.state) {
                UiAction::Submit(text) if engine_task.is_none() => {
                    let trimmed = text.trim().to_string();
                    if trimmed.starts_with('/') {
                        handle_tui_slash(&trimmed, &provider, &config, &mode_flag, &system_prompt,
                            &cwd, &conversation, &skills, &permission, &mut pinned_files, &mut tui).await;
                    } else {
                        let expanded = expand_with_pins(&trimmed, &pinned_files);
                        let pre = conversation.clone();
                        conversation.push(Message { role: Role::User, content: expand_message_content(&expanded) });
                        tui.state.push_user_message(&trimmed);
                        let (ev_tx, ev_rx) = mpsc::unbounded_channel();
                        let task = tokio::spawn({
                            let eng = engine.clone();
                            let conv = conversation.clone();
                            let ti = total_input.clone();
                            let to = total_output.clone();
                            async move { run_engine_tui(&eng, conv, &ti, &to, ev_tx).await }
                        });
                        engine_task = Some((task, ev_rx, pre));
                    }
                }
                UiAction::Quit => { stop_keys.store(true, Ordering::Relaxed); break; }
                _ => {}
            }
        }

        if stop_keys.load(Ordering::Relaxed) { break; }
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    save_session(&session_repo, &conversation).await;
}

async fn handle_tui_slash(
    cmd: &str,
    provider: &Arc<AnthropicProvider>,
    config: &claude_rust_config::Settings,
    mode_flag: &Arc<AtomicU8>,
    system_prompt: &str,
    cwd: &str,
    conversation: &Conversation,
    skills: &[Skill],
    permission: &Arc<ConfigAwarePermissionChecker>,
    pinned_files: &mut Vec<String>,
    tui: &mut TuiApp,
) {
    use super::app_actions::{handle_add, show_skills, copy_last_response, show_files};
    use super::app_help::print_help;

    if cmd == "/quit" || cmd == "/exit" { return; }
    if cmd == "/help" {
        tui.leave_alt();
        print_help(skills);
        let _ = std::io::stdin().read_line(&mut String::new());
        tui.enter_alt();
        return;
    }
    if let Some(path) = cmd.strip_prefix("/add ") { handle_add(path, pinned_files); return; }
    if cmd == "/files" { show_files(pinned_files); return; }
    if cmd == "/copy" { copy_last_response(conversation); return; }
    if cmd == "/skills" { tui.leave_alt(); show_skills(skills); let _ = std::io::stdin().read_line(&mut String::new()); tui.enter_alt(); return; }
    if cmd == "/version" {
        tui.state.push_system_message(format!("claude-rust v{}", env!("CARGO_PKG_VERSION")));
        return;
    }

    match handle_slash_command(cmd, provider, config, mode_flag, system_prompt, cwd, conversation, skills).await {
        Some(CommandAction::Output(text)) => { tui.state.push_system_message(text.trim_end()); }
        Some(CommandAction::Quit) => {}
        _ => { tui.state.push_system_message(format!("Unknown command: {}", cmd.split_whitespace().next().unwrap_or(cmd))); }
    }
    let _ = permission;
}
