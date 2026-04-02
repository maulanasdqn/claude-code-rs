use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use claude_rust_commands::expand_message_content;
use claude_rust_engine::QueryEngine;
use claude_rust_errors::AppError;
use claude_rust_types::{Conversation, EngineEvent, Message, PermissionMode, Role};
use claude_rust_tui::{DisplayMessage, EventHandler, TuiApp, UiAction};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::domain::AppContext;
use crate::infrastructure::terminal::git_branch;

use super::run_engine::run_engine_tui;

type EngineTask = JoinHandle<Result<Conversation, AppError>>;
type EngineSlot = Option<(
    EngineTask,
    mpsc::UnboundedReceiver<EngineEvent>,
    Conversation,
)>;

fn conv_to_tui(conversation: &Conversation) -> Vec<DisplayMessage> {
    conversation
        .messages
        .iter()
        .filter_map(|m| {
            let role = match m.role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            let text = m
                .content
                .iter()
                .filter_map(|b| {
                    if let claude_rust_types::ContentBlock::Text { text } = b {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            if text.is_empty() {
                return None;
            }
            Some(DisplayMessage {
                role: role.to_string(),
                content: text,
                thinking: String::new(),
                tool_uses: Vec::new(),
                is_streaming: false,
            })
        })
        .collect()
}

fn spawn_engine(
    text: &str,
    conversation: &mut Conversation,
    tui: &mut TuiApp,
    engine: &Arc<QueryEngine>,
    total_input: &Arc<AtomicU64>,
    total_output: &Arc<AtomicU64>,
) -> EngineSlot {
    let pre = conversation.clone();
    conversation.push(Message {
        role: Role::User,
        content: expand_message_content(text),
    });
    tui.state.push_user_message(text);
    let (ev_tx, ev_rx) = mpsc::unbounded_channel();
    let task = tokio::spawn({
        let (eng, conv, ti, to) = (
            engine.clone(),
            conversation.clone(),
            total_input.clone(),
            total_output.clone(),
        );
        async move { run_engine_tui(&eng, conv, &ti, &to, ev_tx).await }
    });
    Some((task, ev_rx, pre))
}

pub async fn run_loop(ctx: &AppContext, _system_prompt: String, mut conversation: Conversation) {
    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let model_id = ctx.provider.model_name();

    let mut tui = match TuiApp::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("TUI init failed: {e}");
            return;
        }
    };
    tui.state.model_name = model_id.clone();
    tui.state.git_branch = git_branch();
    tui.state.conversation.messages = conv_to_tui(&conversation);
    ctx.provider.toggle_thinking();

    let (key_tx, mut key_rx) = mpsc::unbounded_channel::<crossterm::event::Event>();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let pause_for_keys = ctx.pause_flag.clone();
    tokio::task::spawn_blocking(move || {
        while !stop2.load(Ordering::Relaxed) {
            if pause_for_keys.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            if crossterm::event::poll(Duration::from_millis(30)).unwrap_or(false) {
                if let Ok(ev) = crossterm::event::read() {
                    if key_tx.send(ev).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut engine_task: EngineSlot = None;

    loop {
        tui.sync_pause(&ctx.pause_flag);

        if let Some((task, ev_rx, _)) = &mut engine_task {
            while let Ok(ev) = ev_rx.try_recv() {
                tui.state.apply_engine_event(ev);
            }
            if task.is_finished() {
                let (task, _, pre) = engine_task.take().unwrap();
                match task.await {
                    Ok(Ok(updated)) => {
                        conversation = updated;
                    }
                    Ok(Err(e)) if e.is_interrupted() => {
                        conversation = pre;
                        if let Some(m) = tui.state.conversation.messages.last_mut() {
                            m.is_streaming = false;
                        }
                    }
                    Ok(Err(e)) => {
                        tui.state
                            .push_system_message(format!("Error: {e}"));
                        conversation = pre;
                    }
                    Err(_) => {
                        conversation = pre;
                    }
                }
                tui.state.is_streaming = false;
            }
        }

        tui.tick_spinner();
        if tui.is_in_alt() {
            tui.draw().ok();
        }

        while let Ok(ev) = key_rx.try_recv() {
            match EventHandler::handle(ev, &mut tui.state) {
                UiAction::Submit(text) if engine_task.is_none() => {
                    let trimmed = text.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match trimmed.as_str() {
                        "/quit" | "/exit" => {
                            stop.store(true, Ordering::Relaxed);
                            break;
                        }
                        "/clear" => {
                            conversation.messages.clear();
                            tui.state.conversation.messages.clear();
                            tui.state.push_system_message("conversation cleared");
                        }
                        "/help" => {
                            tui.state.push_system_message(
                                "/help  /clear  /quit  /version  Shift+Tab to cycle mode",
                            );
                        }
                        "/version" => {
                            tui.state.push_system_message(format!(
                                "claude-rust-cli v{}",
                                env!("CARGO_PKG_VERSION")
                            ));
                        }
                        _ => {
                            engine_task = spawn_engine(
                                &trimmed,
                                &mut conversation,
                                &mut tui,
                                &ctx.engine,
                                &total_input,
                                &total_output,
                            );
                        }
                    }
                }
                UiAction::CyclePermissionMode => {
                    let next = PermissionMode::load(&ctx.mode_flag).next();
                    next.store(&ctx.mode_flag);
                    tui.state.permission_mode = next.label().to_string();
                }
                UiAction::Quit => {
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
                _ => {}
            }
        }

        if stop.load(Ordering::Relaxed) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}
