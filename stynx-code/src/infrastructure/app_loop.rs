use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};
use std::time::Duration;

use stynx_code_commands::expand_message_content;
use stynx_code_engine::{EngineEvent, QueryEngine};
use stynx_code_errors::AppError;
use stynx_code_permission::{ConfigAwarePermissionChecker, PromptBridge, PromptChoice, PromptRequest};
use stynx_code_tools::{QuestionBridge, QuestionRequest, SharedQuestionBridge};

use super::agent_tool::InternTool;
use stynx_code_provider::AnthropicProvider;
use stynx_code_types::{Conversation, Message, PermissionMode, Role};
use stynx_code_tui::state::app_state::SessionSummary;
use stynx_code_tui::state::InputKind;
use stynx_code_tui::{DisplayMessage, EventHandler, PermissionChoice, TuiApp, UiAction};
use tokio::sync::oneshot;
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
            if let stynx_code_types::ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
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
    session_repo: Arc<dyn stynx_code_memory::SessionRepository>,
    provider: Arc<AnthropicProvider>,
    config: stynx_code_config::Settings,
    mode_flag: Arc<AtomicU8>,
    cwd: String,
    system_prompt: String,
    mut conversation: Conversation,
    skills: Vec<Skill>,
    pause_flag: Arc<AtomicBool>,
    permission: Arc<ConfigAwarePermissionChecker>,
    intern_tool: Option<Arc<InternTool>>,
    ask_user_bridge_handle: SharedQuestionBridge,
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
    tui.state.cwd = cwd.clone();
    tui.state.conversation.messages = conv_to_tui(&conversation);
    refresh_sidebar_sessions(&session_repo, &mut tui).await;
    let persisted = stynx_code_tui::persistence::load();
    stynx_code_tui::persistence::apply_to(&mut tui.state, &persisted);
    set_current_model(&model_id);
    provider.toggle_thinking(); // enable thinking by default

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

    let (prompt_bridge, mut prompt_rx) = PromptBridge::new();
    permission.install_prompt_bridge(prompt_bridge);
    let mut pending_prompt: Option<oneshot::Sender<PromptChoice>> = None;

    let (question_bridge, mut question_rx) = QuestionBridge::new();
    ask_user_bridge_handle.set(question_bridge);
    let mut pending_question: Option<oneshot::Sender<Option<String>>> = None;

    loop {
        tui.sync_pause(&pause_flag);

        if let Some((task, ev_rx, _)) = &mut engine_task {
            while let Ok(ev) = ev_rx.try_recv() { tui.state.apply_engine_event(ev); }
            if task.is_finished() {
                let (task, _, pre) = engine_task.take().unwrap();
                // The engine task ended — any pending prompts/questions whose
                // responders belong to that task are now dead and would block
                // the next turn if we kept them around.
                if pending_prompt.take().is_some() || pending_question.take().is_some() {
                    if matches!(&tui.state.modal.active,
                        Some(stynx_code_tui::ModalKind::Permission { .. })
                        | Some(stynx_code_tui::ModalKind::Input { .. }))
                    {
                        tui.state.modal.close();
                    }
                }
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

        if pending_prompt.is_none() {
            if let Ok(req) = prompt_rx.try_recv() {
                let PromptRequest { tool_name, description, responder } = req;
                tui.state.modal.open_permission(tool_name, description);
                pending_prompt = Some(responder);
            }
        }

        // If a pending question existed but the user dismissed the modal,
        // unblock the engine by sending None.
        if pending_question.is_some() && tui.state.modal.active.is_none() {
            if let Some(responder) = pending_question.take() {
                let _ = responder.send(None);
            }
        }

        if pending_question.is_none() && tui.state.modal.active.is_none() {
            if let Ok(req) = question_rx.try_recv() {
                let QuestionRequest { question, responder } = req;
                tui.state.modal.open_input(
                    "Question",
                    question,
                    String::new(),
                    InputKind::AskUserQuestion,
                );
                pending_question = Some(responder);
            }
        }

        tui.tick_spinner();
        if tui.is_in_alt() { tui.draw().ok(); }

        while let Ok(ev) = key_rx.try_recv() {
            match EventHandler::handle(ev, &mut tui.state) {
                UiAction::Submit(text) if engine_task.is_none() => {
                    let trimmed = text.trim().to_string();
                    if let Some(task) = trimmed.strip_prefix("/intern ") {
                        let task = task.trim().to_string();
                        if task.is_empty() {
                            tui.state.push_system_message("usage: /intern <task description>");
                        } else if let Some(intern) = intern_tool.clone() {
                            tui.state.push_system_message(format!("🧑‍🎓 intern working on: {task}"));
                            match intern.run_task(&task).await {
                                Ok(output) => {
                                    tui.state.push_system_message(format!("🧑‍🎓 intern result:\n{output}"));
                                }
                                Err(e) => {
                                    tui.state.push_system_message(format!("intern failed: {e}"));
                                }
                            }
                        } else {
                            tui.state.push_system_message("intern unavailable: set DEEPSEEK_API_KEY (and optionally DEEPSEEK_MODEL / DEEPSEEK_BASE_URL) and restart");
                        }
                        continue;
                    }
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
                UiAction::CyclePermissionMode => {
                    let next = PermissionMode::load(&mode_flag).next();
                    next.store(&mode_flag);
                    tui.state.permission_mode = next.label().to_string();
                }
                UiAction::SelectModel(model_id) => {
                    provider.set_model(&model_id);
                    let effective = provider.model_name();
                    tui.state.model_name = effective.clone();
                    tui.state.push_recent_model(&effective);
                    set_current_model(&effective);
                    tui.state.toasts.success(format!("model → {effective}"));
                }
                UiAction::SelectSession(session_id) => {
                    if session_id == "__current__" {
                        match stynx_code_memory::load_session(&session_repo).await {
                            Ok(Some(loaded)) => {
                                tui.state.conversation.messages = conv_to_tui(&loaded);
                                conversation = loaded;
                                tui.state.push_system_message("session reloaded");
                            }
                            Ok(None) => tui.state.push_system_message("no saved session"),
                            Err(e) => tui.state.push_system_message(format!("session load failed: {e}")),
                        }
                    } else {
                        tui.state.push_system_message(format!("session: {session_id} (multi-session not yet supported)"));
                    }
                }
                UiAction::ToggleSidebar => {
                    tui.state.sidebar.visible = !tui.state.sidebar.visible;
                    let label = if tui.state.sidebar.visible { "sidebar shown" } else { "sidebar hidden" };
                    tui.state.toasts.info(label);
                }
                UiAction::PermissionDecision(choice) => {
                    if let Some(responder) = pending_prompt.take() {
                        let mapped = match choice {
                            PermissionChoice::Once => PromptChoice::AllowOnce,
                            PermissionChoice::Always => PromptChoice::AllowAlways,
                            PermissionChoice::Reject => PromptChoice::Deny,
                        };
                        let _ = responder.send(mapped);
                    }
                }
                UiAction::RunCommand(name) => {
                    match name.as_str() {
                        "session.new" => {
                            conversation = stynx_code_types::Conversation {
                                system: Some(system_prompt.clone()),
                                ..Default::default()
                            };
                            tui.state.conversation.messages.clear();
                            tui.state.total_cost = 0.0;
                            tui.state.total_input = 0;
                            tui.state.total_output = 0;
                            tui.state.sidebar.title = "New session".to_string();
                            tui.state.push_system_message("new session");
                        }
                        "session.compact" => {
                            tui.state.push_system_message("compacting…");
                            let provider_dyn: Arc<dyn stynx_code_types::Provider> = provider.clone();
                            let mut on_event = |_ev: stynx_code_engine::EngineEvent| {};
                            match stynx_code_engine::application::compactor::compact(
                                &provider_dyn,
                                conversation.clone(),
                                &mut on_event,
                            )
                            .await
                            {
                                Ok(compacted) => {
                                    let new_count = compacted.messages.len();
                                    tui.state.conversation.messages = conv_to_tui(&compacted);
                                    conversation = compacted;
                                    save_session(&session_repo, &conversation).await;
                                    tui.state.push_system_message(format!(
                                        "compacted → {new_count} messages"
                                    ));
                                }
                                Err(e) => tui.state.push_system_message(format!("compact failed: {e}")),
                            }
                        }
                        "session.export" => {
                            match export_transcript(&conversation, &cwd).await {
                                Ok(path) => tui.state.toasts.success(format!("exported → {path}")),
                                Err(e) => tui.state.toasts.error(format!("export failed: {e}")),
                            }
                        }
                        "model.cycle_recent" => {
                            if let Some(next) = tui.state.cycle_recent_model() {
                                provider.set_model(&next);
                                let effective = provider.model_name();
                                tui.state.model_name = effective.clone();
                                set_current_model(&effective);
                                tui.state.push_system_message(format!("model → {effective}"));
                            } else {
                                tui.state.push_system_message("no other recent model");
                            }
                        }
                        "help.show" => {
                            let skill_pairs: Vec<(String, String)> = skills
                                .iter()
                                .filter(|s| s.user_invocable)
                                .map(|s| (s.name.clone(), s.description.clone()))
                                .collect();
                            stynx_code_tui::dialogs::open_help(&mut tui.state, &skill_pairs);
                        }
                        "status.show" => {
                            let intern_label = intern_tool.as_ref().map(|_| "deepseek");
                            stynx_code_tui::dialogs::open_status(&mut tui.state, intern_label);
                        }
                        "skills.show" => {
                            let skill_pairs: Vec<(String, String)> = skills
                                .iter()
                                .filter(|s| s.user_invocable)
                                .map(|s| (s.name.clone(), s.description.clone()))
                                .collect();
                            stynx_code_tui::dialogs::open_skill_picker(&mut tui.state, &skill_pairs);
                        }
                        "session.rename" => {
                            let current = tui.state.sidebar.title.clone();
                            tui.state.modal.open_input(
                                "Rename session",
                                "New title:",
                                current,
                                InputKind::SessionRename,
                            );
                        }
                        _ => {}
                    }
                }
                UiAction::InputConfirmed { kind, value } => match kind {
                    InputKind::SessionRename => {
                        tui.state.sidebar.title = value.clone();
                        tui.state.toasts.success(format!("renamed → {value}"));
                    }
                    InputKind::AskUserQuestion => {
                        if let Some(responder) = pending_question.take() {
                            let _ = responder.send(Some(value));
                        }
                    }
                },
                UiAction::Interrupt => {
                    if let Some((task, _, _)) = engine_task.as_ref() {
                        task.abort();
                        tui.state.is_streaming = false;
                        tui.state.toasts.warn("interrupted");
                    }
                    // Drop any in-flight prompts whose engine task just died.
                    pending_prompt.take();
                    pending_question.take();
                    tui.state.modal.close();
                }
                UiAction::Quit => { stop.store(true, Ordering::Relaxed); break; }
                _ => {}
            }
        }

        if stop.load(Ordering::Relaxed) { break; }
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    save_session(&session_repo, &conversation).await;
    stynx_code_tui::persistence::save(&stynx_code_tui::persistence::snapshot(&tui.state));
}

async fn export_transcript(conversation: &Conversation, cwd: &str) -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = format!("transcript-{ts}.md");
    let path = std::path::Path::new(cwd).join(&filename);

    let mut out = String::new();
    out.push_str(&format!("# stynx-code transcript — {ts}\n\n"));
    for msg in &conversation.messages {
        let header = match msg.role {
            Role::User => "## User",
            Role::Assistant => "## Assistant",
        };
        out.push_str(header);
        out.push_str("\n\n");
        for block in &msg.content {
            if let stynx_code_types::ContentBlock::Text { text } = block {
                out.push_str(text);
                out.push_str("\n\n");
            }
        }
    }

    tokio::fs::write(&path, out)
        .await
        .map_err(|e| format!("write failed: {e}"))?;
    Ok(path.display().to_string())
}

async fn refresh_sidebar_sessions(
    repo: &Arc<dyn stynx_code_memory::SessionRepository>,
    tui: &mut TuiApp,
) {
    let ids = match repo.list().await { Ok(v) => v, Err(_) => Vec::new() };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    tui.state.sidebar.sessions = ids
        .into_iter()
        .map(|id| SessionSummary {
            id: if id == "current" { "__current__".to_string() } else { id.clone() },
            title: if id == "current" { "Active session".to_string() } else { id },
            updated_at: now,
            pinned: false,
        })
        .collect();
    if let Some(first) = tui.state.sidebar.sessions.first() {
        tui.state.sidebar.session_id = first.id.clone();
        if tui.state.sidebar.title == "New session" {
            tui.state.sidebar.title = first.title.clone();
        }
    }
}

async fn handle_tui_slash(
    cmd: &str,
    provider: &Arc<AnthropicProvider>,
    config: &stynx_code_config::Settings,
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
        "/version" => { tui.state.push_system_message(format!("stynx-code v{}", env!("CARGO_PKG_VERSION"))); return None; }
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
