mod display;
mod sidebar;
mod slash_handler;

use std::collections::VecDeque;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};
use std::time::{Duration, Instant};

use stynx_code_engine::QueryEngine;
use stynx_code_permission::{ConfigAwarePermissionChecker, PromptBridge, PromptChoice, PromptRequest};
use stynx_code_tools::{QuestionBridge, QuestionRequest, SharedQuestionBridge};

use super::agent_tool::InternTool;
use stynx_code_types::{Conversation, PermissionMode, Provider};
use stynx_code_tui::state::InputKind;
use stynx_code_tui::{EventHandler, PermissionChoice, TuiApp, UiAction};
use tokio::sync::oneshot;
use tokio::sync::mpsc;

use super::app_actions::{expand_with_pins, save_session};
use super::intern_bench;
use super::command_types::CommandAction;
use super::skills::Skill;
use super::terminal::set_current_model;

use display::{conv_to_tui, export_transcript, fmt_elapsed, fmt_tokens};
use sidebar::{spawn_engine, refresh_sidebar_sessions, EngineSlot};


#[allow(clippy::too_many_arguments)]
pub async fn run_loop(
    engine: Arc<QueryEngine>,
    _conductor_engine: Arc<QueryEngine>,
    _reflect_engine: Arc<QueryEngine>,
    session_repo: Arc<dyn stynx_code_memory::SessionRepository>,
    provider: Arc<dyn Provider>,
    anthropic: Option<Arc<stynx_code_provider::AnthropicProvider>>,
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
    if anthropic.is_some() {
        provider.toggle_thinking();
    }

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
    let mut last_event_at: Option<Instant> = None;
    let mut tokens_at_start: (u64, u64) = (0, 0);
    let perm = permission.clone();

    let (prompt_bridge, mut prompt_rx) = PromptBridge::new();
    permission.install_prompt_bridge(prompt_bridge);
    let mut pending_prompt: Option<oneshot::Sender<PromptChoice>> = None;

    let (question_bridge, mut question_rx) = QuestionBridge::new();
    ask_user_bridge_handle.set(question_bridge);
    let mut pending_question: Option<oneshot::Sender<Option<String>>> = None;

    // Prompts the user submits while the engine is busy. Replayed one at a
    // time as each turn completes, so a prompt typed mid-stream is never lost.
    let mut queued_prompts: VecDeque<String> = VecDeque::new();
    // Actions to process this iteration that didn't come from a key event
    // (e.g. a queued prompt being replayed once the engine frees up).
    let mut deferred: VecDeque<UiAction> = VecDeque::new();

    loop {
        tui.sync_pause(&pause_flag);

        if let Some((task, ev_rx, _)) = &mut engine_task {
            let had_event = {
                let mut any = false;
                while let Ok(ev) = ev_rx.try_recv() {
                    tui.state.apply_engine_event(ev);
                    any = true;
                }
                any
            };
            if had_event { last_event_at = Some(Instant::now()); }

            if let Some(started) = engine_started {
                tui.state.elapsed_secs = started.elapsed().as_secs();
            }

            if tui.state.is_streaming && !tui.state.stale_warned {
                let silence = last_event_at.map(|t| t.elapsed()).unwrap_or(Duration::ZERO);
                if silence > Duration::from_secs(30) {
                    tui.state.stale_warned = true;
                    tui.state.toasts.warn("model silent for 30s — press Esc to interrupt");
                }
            }

            if task.is_finished() {
                let (task, _, pre) = engine_task.take().unwrap();

                if pending_prompt.take().is_some() || pending_question.take().is_some() {
                    if matches!(&tui.state.modal.active,
                        Some(stynx_code_tui::ModalKind::Permission { .. })
                        | Some(stynx_code_tui::ModalKind::Input { .. }))
                    {
                        tui.state.modal.close();
                    }
                }
                tui.state.is_pending = false;
                tui.state.stale_warned = false;
                tui.state.elapsed_secs = 0;
                last_event_at = None;
                let elapsed = engine_started.take()
                    .map(|t| t.elapsed())
                    .unwrap_or(std::time::Duration::ZERO);
                let (start_in, start_out) = std::mem::take(&mut tokens_at_start);
                let in_delta = total_input.load(Ordering::Relaxed).saturating_sub(start_in);
                let out_delta = total_output.load(Ordering::Relaxed).saturating_sub(start_out);
                match task.await {
                    Ok(Ok(updated)) => {
                        if updated.messages.len() <= conversation.messages.len() {
                            // Mid-run compaction rewrote history: archive the full
                            // transcript under the current session id, then continue
                            // in a fresh session so the original is never lost.
                            save_session(&session_repo, &conversation).await;
                            if let Ok(id) = session_repo.new_session_id().await {
                                let _ = session_repo.set_current(&id).await;
                                tui.state.sidebar.session_id = id;
                            }
                            refresh_sidebar_sessions(&session_repo, &mut tui).await;
                        }
                        conversation = updated;
                        save_session(&session_repo, &conversation).await;
                        let sid = session_repo.current().await.ok().flatten();
                        stynx_code_truncus::capture(sid.as_deref(), &cwd, &conversation, false).await;
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

                // The engine is now idle — replay the next queued prompt, if any.
                if let Some(next) = queued_prompts.pop_front() {
                    deferred.push_back(UiAction::Submit(next));
                }
            }
        }

        if pending_prompt.is_none() {
            if let Ok(req) = prompt_rx.try_recv() {
                let PromptRequest { tool_name, description, responder } = req;
                tui.state.modal.open_permission(tool_name, description);
                pending_prompt = Some(responder);
            }
        }

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

        loop {
            // Live key events take priority; once they're drained, replay any
            // deferred actions (e.g. a queued prompt freed up by turn completion).
            let action = match key_rx.try_recv() {
                Ok(ev) => EventHandler::handle(ev, &mut tui.state),
                Err(_) => match deferred.pop_front() {
                    Some(a) => a,
                    None => break,
                },
            };
            match action {
                UiAction::Submit(text) if engine_task.is_none() => {
                    let trimmed = text.trim().to_string();
                    if let Some(rest) = trimmed.strip_prefix("/intern-bench").or_else(|| trimmed.strip_prefix("/bench")) {
                        let rest = rest.trim();
                        if intern_tools.is_empty() {
                            tui.state.push_system_message(
                                "no interns configured. add `interns` to .stynx/settings.json, \
or set DEEPSEEK_API_KEY / OPENROUTER_API_KEY in .env and restart.",
                            );
                            continue;
                        }
                        let filter = if rest.is_empty() { None } else { Some(rest) };
                        let count = match filter {
                            Some(f) => intern_tools.iter().filter(|t| t.label().eq_ignore_ascii_case(f)).count(),
                            None => intern_tools.len(),
                        };
                        tui.state.push_system_message(format!(
                            "🏁 benchmarking {count} intern(s) on {} tasks — this blocks the UI for a few minutes \
(auto-accept is forced on during the run, then restored)…",
                            intern_bench::BENCH_TASKS.len(),
                        ));
                        tui.draw().ok();

                        // Force a non-prompting mode: the loop is blocked while the bench runs,
                        // so a permission prompt would deadlock it. Restore the prior mode after.
                        let prev_mode = PermissionMode::load(&mode_flag);
                        PermissionMode::AutoAccept.store(&mode_flag);

                        let (summary, markdown) = intern_bench::run_intern_bench(
                            &intern_tools,
                            provider.clone(),
                            permission.clone(),
                            mode_flag.clone(),
                            config.hooks.clone(),
                            filter,
                        ).await;

                        prev_mode.store(&mode_flag);

                        let report_path = std::path::Path::new(&cwd).join(".stynx").join("intern-bench.md");
                        if let Some(dir) = report_path.parent() {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        if let Err(e) = std::fs::write(&report_path, &markdown) {
                            tui.state.push_system_message(format!("(couldn't write report file: {e})"));
                        }

                        tui.state.push_system_message(summary);
                        continue;
                    }
                    if let Some(rest) = trimmed.strip_prefix("/intern") {
                        let rest = rest.trim();
                        if intern_tools.is_empty() {
                            tui.state.push_system_message(
                                "no interns configured. add `interns` to .stynx/settings.json, \
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
                        let action = slash_handler::handle_tui_slash(&trimmed, &provider, anthropic.as_deref(), &config, &mode_flag,
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
                                last_event_at = Some(Instant::now());
                                tokens_at_start = (total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
                                tui.state.is_pending = true;
                                tui.state.stale_warned = false;
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
                        last_event_at = Some(Instant::now());
                        tokens_at_start = (total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
                        tui.state.is_pending = true;
                        tui.state.stale_warned = false;
                    }
                }
                UiAction::Submit(text) => {
                    // Engine is busy — queue the prompt instead of dropping it.
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        queued_prompts.push_back(trimmed);
                        tui.state.toasts.info(format!(
                            "queued — will send after current response ({} pending)",
                            queued_prompts.len(),
                        ));
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
                            match stynx_code_compact::FullCompactor::new()
                                .compact(&conversation, provider_dyn.as_ref())
                                .await
                            {
                                Ok(compacted) => {
                                    // Archive the full transcript under the current
                                    // session, then continue in a fresh one.
                                    save_session(&session_repo, &conversation).await;
                                    if let Ok(id) = session_repo.new_session_id().await {
                                        let _ = session_repo.set_current(&id).await;
                                        tui.state.sidebar.session_id = id;
                                    }
                                    let new_count = compacted.messages.len();
                                    tui.state.conversation.messages = conv_to_tui(&compacted);
                                    conversation = compacted;
                                    save_session(&session_repo, &conversation).await;
                                    refresh_sidebar_sessions(&session_repo, &mut tui).await;
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
                        tui.state.is_pending = false;
                        tui.state.stale_warned = false;
                        tui.state.elapsed_secs = 0;
                        tui.state.is_paused = true;
                        tui.state.sub_agents.clear();
                        last_event_at = None;
                        tui.state.toasts.warn("interrupted");
                    }

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
    let sid = session_repo.current().await.ok().flatten();
    stynx_code_truncus::capture(sid.as_deref(), &cwd, &conversation, true).await;
    stynx_code_tui::persistence::save(&stynx_code_tui::persistence::snapshot(&tui.state));
}


