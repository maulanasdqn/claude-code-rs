use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};
use std::time::Duration;

use claude_rust_commands::expand_message_content;
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::AppError;
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};
use claude_rust_tui::{DisplayMessage, EventHandler, TuiApp, UiAction};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::app_actions::{expand_with_pins, save_session};
use super::command_handler::handle_slash_command;
use super::command_types::CommandAction;
use super::run_engine::run_engine_tui;
use super::skills::Skill;
use super::terminal::set_current_model;

type EngineTask = JoinHandle<Result<Conversation, AppError>>;
type EngineSlot = Option<(EngineTask, mpsc::UnboundedReceiver<EngineEvent>, Conversation)>;

fn conv_to_tui(conversation: &Conversation) -> Vec<DisplayMessage> {
    conversation.messages.iter().filter_map(|m| {
        let role = match m.role { Role::User => "user", Role::Assistant => "assistant" };
        let text = m.content.iter().filter_map(|b| {
            if let claude_rust_types::ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
        }).collect::<Vec<_>>().join("");
        if text.is_empty() { return None; }
        Some(DisplayMessage { role: role.to_string(), content: text, thinking: String::new(), tool_uses: Vec::new(), is_streaming: false })
    }).collect()
}

fn spawn_engine(
    text: &str, conversation: &mut Conversation, tui: &mut TuiApp,
    engine: &Arc<QueryEngine>, total_input: &Arc<AtomicU64>, total_output: &Arc<AtomicU64>,
) -> EngineSlot {
    let pre = conversation.clone();
    conversation.push(Message { role: Role::User, content: expand_message_content(text) });
    tui.state.push_user_message(text);
    let (ev_tx, ev_rx) = mpsc::unbounded_channel();
    let task = tokio::spawn({
        let (eng, conv, ti, to) = (engine.clone(), conversation.clone(), total_input.clone(), total_output.clone());
        async move { run_engine_tui(&eng, conv, &ti, &to, ev_tx).await }
    });
    Some((task, ev_rx, pre))
}

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
    tui.state.conversation.messages = conv_to_tui(&conversation);
    set_current_model(&model_id);

    let (key_tx, mut key_rx) = mpsc::unbounded_channel::<crossterm::event::Event>();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let pause_for_keys = pause_flag.clone();
    tokio::task::spawn_blocking(move || {
        while !stop2.load(Ordering::Relaxed) {
            if pause_for_keys.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            if crossterm::event::poll(Duration::from_millis(30)).unwrap_or(false) {
                if let Ok(ev) = crossterm::event::read() { if key_tx.send(ev).is_err() { break; } }
            }
        }
    });

    let mut engine_task: EngineSlot = None;
    let perm = permission.clone();

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
                        if let Some(m) = tui.state.conversation.messages.last_mut() { m.is_streaming = false; }
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
                        let action = handle_tui_slash(&trimmed, &provider, &config, &mode_flag,
                            &system_prompt, &cwd, &conversation, &skills, &mut pinned_files, &mut tui).await;
                        match action {
                            Some(CommandAction::ReplaceConversation(c)) => {
                                tui.state.conversation.messages = conv_to_tui(&c);
                                conversation = c;
                            }
                            Some(CommandAction::SendToEngine(msg, tools)) => {
                                if !tools.is_empty() { perm.set_skill_allow_rules(tools); }
                                engine_task = spawn_engine(&msg, &mut conversation, &mut tui, &engine, &total_input, &total_output);
                                perm.clear_skill_allow_rules();
                            }
                            Some(CommandAction::Output(t)) => tui.state.push_system_message(t.trim_end()),
                            Some(CommandAction::Quit) => { stop.store(true, Ordering::Relaxed); break; }
                            Some(CommandAction::Continue) | None => {}
                        }
                    } else {
                        let expanded = expand_with_pins(&trimmed, &pinned_files);
                        engine_task = spawn_engine(&expanded, &mut conversation, &mut tui, &engine, &total_input, &total_output);
                    }
                }
                UiAction::Quit => { stop.store(true, Ordering::Relaxed); break; }
                _ => {}
            }
        }

        if stop.load(Ordering::Relaxed) { break; }
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
    pinned_files: &mut Vec<String>,
    tui: &mut TuiApp,
) -> Option<CommandAction> {
    use super::app_actions::{copy_last_response, handle_add, show_files, show_skills};
    use super::app_help::print_help;

    match cmd {
        "/quit" | "/exit" => return Some(CommandAction::Quit),
        "/version" => { tui.state.push_system_message(format!("claude-rust v{}", env!("CARGO_PKG_VERSION"))); return None; }
        "/files" => { show_files(pinned_files); return None; }
        "/copy" => { copy_last_response(conversation); return None; }
        "/help" => {
            tui.leave_alt(); print_help(skills); let _ = std::io::stdin().read_line(&mut String::new()); tui.enter_alt();
            return None;
        }
        "/skills" => {
            tui.leave_alt(); show_skills(skills); let _ = std::io::stdin().read_line(&mut String::new()); tui.enter_alt();
            return None;
        }
        _ => {}
    }
    if let Some(path) = cmd.strip_prefix("/add ") { handle_add(path, pinned_files); return None; }

    tui.leave_alt();
    let result = handle_slash_command(cmd, provider, config, mode_flag, system_prompt, cwd, conversation, skills).await;
    tui.enter_alt();
    result
}
