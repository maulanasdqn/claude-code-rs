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
    intern_tools: Vec<Arc<InternTool>>,
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
    let mut engine_started: Option<std::time::Instant> = None;
    let mut tokens_at_start: (u64, u64) = (0, 0);
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
                let elapsed = engine_started.take()
                    .map(|t| t.elapsed())
                    .unwrap_or(std::time::Duration::ZERO);
                let (start_in, start_out) = std::mem::take(&mut tokens_at_start);
                let in_delta = total_input.load(Ordering::Relaxed).saturating_sub(start_in);
                let out_delta = total_output.load(Ordering::Relaxed).saturating_sub(start_out);
                match task.await {
                    Ok(Ok(updated)) => {
                        conversation = updated;
                        save_session(&session_repo, &conversation).await;
                        tui.state.toasts.success(format!(
                            "done · {} · {} in / {} out",
                            fmt_elapsed(elapsed),
                            fmt_tokens(in_delta),
                            fmt_tokens(out_delta),
                        ));
                    }
                    Ok(Err(e)) if e.is_interrupted() => {
                        conversation = pre;
                        if let Some(m) = tui.state.conversation.messages.last_mut() { m.is_streaming = false; }
                    }
                    Ok(Err(e)) => {
                        tui.state.push_system_message(format!("Error: {e}"));
                        tui.state.toasts.error(format!("failed · {}", fmt_elapsed(elapsed)));
                        conversation = pre;
                    }
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
                    if let Some(rest) = trimmed.strip_prefix("/intern") {
                        let rest = rest.trim();
                        if intern_tools.is_empty() {
                            tui.state.push_system_message(
                                "no interns configured. add `interns` to .claude/settings.json, \
or set DEEPSEEK_API_KEY / OPENROUTER_API_KEY in .env and restart.",
                            );
                            continue;
                        }
                        if rest.is_empty() {
                            let names: Vec<String> = intern_tools.iter()
                                .map(|t| t.label().to_string()).collect();
                            tui.state.push_system_message(format!(
                                "usage: /intern [<name>] <task description>\navailable interns: {}",
                                names.join(", "),
                            ));
                            continue;
                        }
                        // Try `/intern <name> <task>`: match name against intern labels.
                        let (intern, task) = match rest.split_once(' ') {
                            Some((first, tail)) => {
                                let first_t = first.trim();
                                let pick = intern_tools.iter()
                                    .find(|t| t.label().eq_ignore_ascii_case(first_t))
                                    .cloned();
                                match pick {
                                    Some(t) => (t, tail.trim().to_string()),
                                    None => (intern_tools[0].clone(), rest.to_string()),
                                }
                            }
                            None => (intern_tools[0].clone(), rest.to_string()),
                        };
                        if task.is_empty() {
                            tui.state.push_system_message("usage: /intern [<name>] <task description>");
                            continue;
                        }
                        tui.state.push_system_message(format!(
                            "🧑‍🎓 {label} intern working on: {task}",
                            label = intern.label(),
                        ));
                        match intern.run_task(&task).await {
                            Ok(output) => {
                                tui.state.push_system_message(format!(
                                    "🧑‍🎓 {label} intern result:\n{output}",
                                    label = intern.label(),
                                ));
                            }
                            Err(e) => {
                                tui.state.push_system_message(format!(
                                    "{label} intern failed: {e}",
                                    label = intern.label(),
                                ));
                            }
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
                                engine_started = Some(std::time::Instant::now());
                                tokens_at_start = (total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
                                perm.clear_skill_allow_rules();
                            }
                            Some(CommandAction::Output(t)) => tui.state.push_system_message(t.trim_end()),
                            Some(CommandAction::Quit) => { stop.store(true, Ordering::Relaxed); break; }
                            Some(CommandAction::Continue) | None => {}
                        }
                    } else {
                        let expanded = expand_with_pins(&trimmed, &pinned_files);
                        engine_task = spawn_engine(&expanded, &mut conversation, &mut tui, &engine, &total_input, &total_output);
                        engine_started = Some(std::time::Instant::now());
                        tokens_at_start = (total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
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
                    match session_repo.load(&session_id).await {
                        Ok(Some(loaded)) => {
                            tui.state.conversation.messages = conv_to_tui(&loaded);
                            conversation = loaded;
                            let _ = session_repo.set_current(&session_id).await;
                            tui.state.sidebar.session_id = session_id.clone();
                            refresh_sidebar_sessions(&session_repo, &mut tui).await;
                            tui.state.toasts.success(format!("switched → {}", &tui.state.sidebar.title));
                        }
                        Ok(None) => tui.state.toasts.warn("session not found"),
                        Err(e) => tui.state.toasts.error(format!("load failed: {e}")),
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
                            match session_repo.new_session_id().await {
                                Ok(id) => {
                                    let _ = session_repo.set_current(&id).await;
                                    if let Err(e) = session_repo.save(Some(&id), &conversation).await {
                                        tracing::warn!("save new session failed: {e}");
                                    }
                                    tui.state.sidebar.title = "New session".to_string();
                                    tui.state.sidebar.session_id = id;
                                    refresh_sidebar_sessions(&session_repo, &mut tui).await;
                                    tui.state.toasts.success("new session");
                                }
                                Err(e) => {
                                    tui.state.toasts.error(format!("new session failed: {e}"));
                                }
                            }
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
                            let intern_labels: Vec<String> = intern_tools.iter()
                                .map(|t| t.label().to_string()).collect();
                            stynx_code_tui::dialogs::open_status(&mut tui.state, &intern_labels);
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
                        let id = tui.state.sidebar.session_id.clone();
                        if !id.is_empty() {
                            if let Err(e) = session_repo.rename(&id, &value).await {
                                tracing::warn!("rename failed: {e}");
                            }
                        }
                        tui.state.sidebar.title = value.clone();
                        refresh_sidebar_sessions(&session_repo, &mut tui).await;
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
    let summaries = repo.list().await.unwrap_or_default();
    let current = repo.current().await.ok().flatten();
    tui.state.sidebar.sessions = summaries
        .iter()
        .map(|s| SessionSummary {
            id: s.id.clone(),
            title: s.title.clone(),
            updated_at: s.updated_at,
            pinned: false,
        })
        .collect();
    if let Some(id) = current {
        if let Some(s) = summaries.iter().find(|s| s.id == id) {
            tui.state.sidebar.session_id = s.id.clone();
            tui.state.sidebar.title = s.title.clone();
        } else if let Some(first) = summaries.first() {
            tui.state.sidebar.session_id = first.id.clone();
            tui.state.sidebar.title = first.title.clone();
        }
    } else if let Some(first) = summaries.first() {
        tui.state.sidebar.session_id = first.id.clone();
        tui.state.sidebar.title = first.title.clone();
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

fn fmt_elapsed(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let ms = d.subsec_millis();
    if secs >= 60 {
        let m = secs / 60;
        let s = secs % 60;
        format!("{m}m {s}s")
    } else if secs >= 10 {
        format!("{secs}s")
    } else {
        format!("{secs}.{:01}s", ms / 100)
    }
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}k", n as f64 / 1_000.0) }
    else { n.to_string() }
}
