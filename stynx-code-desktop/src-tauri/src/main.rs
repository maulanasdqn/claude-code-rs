#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use serde::{Deserialize, Serialize};
use stynx_code_app::{AppOptions, build_app};
use stynx_code_engine::QueryEngine;
use stynx_code_memory::SessionRepository;
use stynx_code_permission::{PromptBridge, PromptChoice, PromptRequest};
use stynx_code_provider::AnthropicProvider;
use stynx_code_tools::{QuestionBridge, QuestionRequest};
use stynx_code_types::{
    ContentBlock, Conversation, EngineEvent, Message, PermissionMode, Provider, Role,
};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{Mutex, oneshot};

const ENGINE_EVENT: &str = "engine-event";

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
enum UiEvent {
    TextDelta { text: String },
    ThinkingDelta { text: String },
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    ToolOutput { name: String, chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    Usage { input_tokens: u64, output_tokens: u64 },
    TurnComplete,
    Compacted { original_turns: u32 },
    ModeChanged { mode: String },
    HookOutput { source: String, output: String },
    SubAgentProgress { label: String, summary: String },
    SubAgentDone { label: String },
    RetryNotice { attempt: u32, max_attempts: u32, delay_ms: u64, message: String },
    Error { message: String },
    PermissionRequest { id: u64, tool_name: String, description: String },
    AskUserRequest { id: u64, question: String },
    WorkspaceMessageRequest { id: u64, target: String, task: String },
    Idle,
}

impl From<EngineEvent> for UiEvent {
    fn from(event: EngineEvent) -> Self {
        match event {
            EngineEvent::TextDelta(text) => UiEvent::TextDelta { text },
            EngineEvent::ThinkingDelta(text) => UiEvent::ThinkingDelta { text },
            EngineEvent::ToolStart { name, id } => UiEvent::ToolStart { name, id },
            EngineEvent::ToolInput { json_chunk } => UiEvent::ToolInput { json_chunk },
            EngineEvent::ToolOutput { name, chunk } => UiEvent::ToolOutput { name, chunk },
            EngineEvent::ToolResult { name, output, is_error } => {
                UiEvent::ToolResult { name, output, is_error }
            }
            EngineEvent::Usage { input_tokens, output_tokens } => {
                UiEvent::Usage { input_tokens, output_tokens }
            }
            EngineEvent::TurnComplete => UiEvent::TurnComplete,
            EngineEvent::Compacted { original_turns } => {
                UiEvent::Compacted { original_turns: original_turns as u32 }
            }
            EngineEvent::ModeChanged { mode } => UiEvent::ModeChanged { mode: mode.to_string() },
            EngineEvent::HookOutput { source, output } => UiEvent::HookOutput { source, output },
            EngineEvent::SubAgentProgress { label, summary } => {
                UiEvent::SubAgentProgress { label, summary }
            }
            EngineEvent::SubAgentDone { label } => UiEvent::SubAgentDone { label },
            EngineEvent::RetryNotice { attempt, max_attempts, delay_ms, message } => {
                UiEvent::RetryNotice { attempt, max_attempts, delay_ms, message }
            }
            EngineEvent::Error(message) => UiEvent::Error { message },
        }
    }
}

fn emit(app: &AppHandle, event: UiEvent) -> bool {
    app.emit(ENGINE_EVENT, event).is_ok()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    model_id: String,
    current_provider: String,
    main_providers: Vec<String>,
    claude_available: bool,
    claude_models: Vec<String>,
    deepseek_models: Vec<String>,
    interns: Vec<InternInfo>,
    thinking_enabled: bool,
    workspace_path: String,
    project_name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct InternInfo {
    name: String,
    provider: String,
    model: String,
    description: String,
    key_env: String,
    available: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionSummary {
    id: String,
    title: String,
    updated_at: u64,
    message_count: u32,
}

#[derive(Serialize)]
struct Turn {
    role: String,
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImagePayload {
    media_type: String,
    data: String,
}

fn current_interns() -> Vec<InternInfo> {
    stynx_code_app::list_interns_current()
        .into_iter()
        .map(|intern| InternInfo {
            name: intern.name,
            provider: intern.provider,
            model: intern.model,
            description: intern.description,
            key_env: intern.key_env,
            available: intern.available,
        })
        .collect()
}

type PromptResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<PromptChoice>>>>;
type QuestionResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<Option<String>>>>>;
type WorkspaceResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<String>>>>;

struct Session {
    engine: Arc<QueryEngine>,
    conversation: Arc<Mutex<Conversation>>,
    mode_flag: Arc<AtomicU8>,
    provider: Arc<dyn Provider>,
    provider_label: String,
    system_prompt: String,
    workspace_path: String,
    anthropic: Option<Arc<AnthropicProvider>>,
    session_repo: Arc<dyn SessionRepository>,
    current_session_id: Arc<StdMutex<Option<String>>>,
    prompt_responders: PromptResponders,
    question_responders: QuestionResponders,
    workspace_responders: WorkspaceResponders,
    current_task: Arc<StdMutex<Option<tokio::task::AbortHandle>>>,
}

#[derive(Default)]
struct SessionSlot(Mutex<Option<Arc<Session>>>);

async fn active(slot: &State<'_, SessionSlot>) -> Result<Arc<Session>, String> {
    slot.0
        .lock()
        .await
        .clone()
        .ok_or_else(|| "no active session — call init_session first".to_string())
}

#[tauri::command]
async fn init_session(
    app: AppHandle,
    slot: State<'_, SessionSlot>,
    workspace_path: Option<String>,
    provider: Option<String>,
) -> Result<SessionInfo, String> {
    let path = match workspace_path {
        Some(p) if !p.trim().is_empty() => p,
        _ => std::env::current_dir()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string(),
    };

    let mut options = AppOptions::new(path.clone());
    options.provider_override = provider;
    let handles = build_app(options).await.map_err(|message| message)?;

    let prompt_responders: PromptResponders = Arc::new(StdMutex::new(HashMap::new()));
    let question_responders: QuestionResponders = Arc::new(StdMutex::new(HashMap::new()));
    let workspace_responders: WorkspaceResponders = Arc::new(StdMutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicU64::new(1));

    let (prompt_bridge, mut prompt_rx) = PromptBridge::new();
    handles.permission.install_prompt_bridge(prompt_bridge);
    {
        let app = app.clone();
        let responders = prompt_responders.clone();
        let next_id = next_id.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = prompt_rx.recv().await {
                let PromptRequest { tool_name, description, responder } = request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered =
                    emit(&app, UiEvent::PermissionRequest { id, tool_name, description });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send(PromptChoice::Deny);
                }
            }
        });
    }

    let (question_bridge, mut question_rx) = QuestionBridge::new();
    handles.ask_user_bridge.set(question_bridge);
    {
        let app = app.clone();
        let responders = question_responders.clone();
        let next_id = next_id.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = question_rx.recv().await {
                let QuestionRequest { question, responder } = request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered = emit(&app, UiEvent::AskUserRequest { id, question });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send(None);
                }
            }
        });
    }

    let (workspace_bridge, mut workspace_rx) = stynx_code_app::WorkspaceBridge::new();
    handles.workspace_bridge.set(workspace_bridge);
    {
        let app = app.clone();
        let responders = workspace_responders.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(request) = workspace_rx.recv().await {
                let stynx_code_app::workspace_bridge::WorkspaceRequest { target, task, responder } =
                    request;
                let id = next_id.fetch_add(1, Ordering::Relaxed);
                responders.lock().unwrap().insert(id, responder);
                let delivered = emit(&app, UiEvent::WorkspaceMessageRequest { id, target, task });
                if !delivered
                    && let Some(responder) = responders.lock().unwrap().remove(&id)
                {
                    let _ = responder.send("(no UI connected to route the message)".to_string());
                }
            }
        });
    }

    let system_prompt = handles.system_prompt;
    let anthropic = handles.anthropic.clone();
    if let Some(provider) = &anthropic {
        provider.set_max_tokens(16000);
        provider.set_thinking_budget(8000);
    }

    let session = Arc::new(Session {
        engine: handles.engine,
        conversation: Arc::new(Mutex::new(Conversation {
            system: Some(system_prompt.clone()),
            messages: Vec::new(),
        })),
        mode_flag: handles.mode_flag,
        provider: handles.provider,
        provider_label: handles.provider_label,
        system_prompt,
        workspace_path: path.clone(),
        anthropic,
        session_repo: handles.session_repo,
        current_session_id: Arc::new(StdMutex::new(None)),
        prompt_responders,
        question_responders,
        workspace_responders,
        current_task: Arc::new(StdMutex::new(None)),
    });

    let info = session_info(&session);
    *slot.0.lock().await = Some(session);
    Ok(info)
}

fn session_info(session: &Session) -> SessionInfo {
    let project_name = std::path::Path::new(&session.workspace_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    SessionInfo {
        model_id: session.provider.model_name(),
        current_provider: session.provider_label.clone(),
        main_providers: stynx_code_app::list_main_providers(),
        claude_available: session.anthropic.is_some(),
        claude_models: vec![
            "claude-opus-4-8".to_string(),
            "claude-sonnet-4-6".to_string(),
            "claude-haiku-4-5-20251001".to_string(),
        ],
        deepseek_models: vec!["deepseek-v4-pro".to_string(), "deepseek-v4-flash".to_string()],
        interns: current_interns(),
        thinking_enabled: session
            .anthropic
            .as_ref()
            .map(|provider| provider.thinking_enabled())
            .unwrap_or(false),
        workspace_path: session.workspace_path.clone(),
        project_name,
    }
}

#[tauri::command]
async fn get_session_info(slot: State<'_, SessionSlot>) -> Result<SessionInfo, String> {
    Ok(session_info(active(&slot).await?.as_ref()))
}

#[tauri::command]
async fn send_message(
    app: AppHandle,
    slot: State<'_, SessionSlot>,
    text: String,
    images: Option<Vec<ImagePayload>>,
) -> Result<(), String> {
    let session = active(&slot).await?;
    let mut content: Vec<ContentBlock> = images
        .unwrap_or_default()
        .into_iter()
        .map(|image| ContentBlock::Image { media_type: image.media_type, data: image.data })
        .collect();
    if !text.trim().is_empty() {
        content.extend(stynx_code_commands::expand_message_content(&text));
    }
    if content.is_empty() {
        return Err("empty message".to_string());
    }
    dispatch(app, session, Message { role: Role::User, content });
    Ok(())
}

fn dispatch(app: AppHandle, session: Arc<Session>, message: Message) {
    let engine = session.engine.clone();
    let conversation = session.conversation.clone();
    let session_repo = session.session_repo.clone();
    let current_session_id = session.current_session_id.clone();
    let workspace_path = session.workspace_path.clone();
    let sink = app.clone();

    let handle = tauri::async_runtime::spawn(async move {
        let mut guard = conversation.lock().await;
        guard.push(message);
        let snapshot = guard.clone();

        let result = engine
            .run(snapshot, move |event| {
                let _ = emit(&sink, UiEvent::from(event));
            })
            .await;

        match result {
            Ok(updated) => {
                let compacted_mid_run = updated.messages.len() <= guard.messages.len();
                let existing = current_session_id.lock().unwrap().clone();
                if compacted_mid_run
                    && let Some(old_id) = &existing
                    && let Err(error) = session_repo.save(Some(old_id), &guard).await
                {
                    tracing::warn!("transcript archive before compact failed: {error}");
                }
                let target = match existing {
                    Some(id) if !compacted_mid_run => Some(id),
                    _ => session_repo.new_session_id().await.ok(),
                };
                match session_repo.save(target.as_deref(), &updated).await {
                    Ok(id) => *current_session_id.lock().unwrap() = Some(id),
                    Err(error) => tracing::warn!("session save failed: {error}"),
                }
                let captured_id = current_session_id.lock().unwrap().clone();
                stynx_code_truncus::capture(captured_id.as_deref(), &workspace_path, &updated, false)
                    .await;
                *guard = updated;
                let _ = emit(&app, UiEvent::Idle);
            }
            Err(error) => {
                let _ = emit(&app, UiEvent::Error { message: error.to_string() });
                let _ = emit(&app, UiEvent::Idle);
            }
        }
    });

    *session.current_task.lock().unwrap() = Some(handle.inner().abort_handle());
}

#[tauri::command]
async fn cancel(app: AppHandle, slot: State<'_, SessionSlot>) -> Result<(), String> {
    let session = active(&slot).await?;
    if let Some(handle) = session.current_task.lock().unwrap().take() {
        handle.abort();
    }
    let _ = emit(&app, UiEvent::Idle);
    Ok(())
}

#[tauri::command]
async fn set_model(slot: State<'_, SessionSlot>, model: String) -> Result<String, String> {
    let session = active(&slot).await?;
    session.provider.set_model(&model);
    Ok(session.provider.model_name())
}

#[tauri::command]
async fn set_mode(slot: State<'_, SessionSlot>, mode: String) -> Result<(), String> {
    let session = active(&slot).await?;
    let value = match mode.as_str() {
        "auto" | "auto_accept" => PermissionMode::AutoAccept,
        "plan" => PermissionMode::Plan,
        "bypass" => PermissionMode::Bypass,
        _ => PermissionMode::Normal,
    };
    session.mode_flag.store(value as u8, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn set_thinking(slot: State<'_, SessionSlot>, enabled: bool) -> Result<(), String> {
    let session = active(&slot).await?;
    let Some(provider) = &session.anthropic else { return Ok(()) };
    if enabled {
        provider.set_max_tokens(16000);
        provider.set_thinking_budget(8000);
    } else if provider.thinking_enabled() {
        provider.toggle_thinking();
    }
    Ok(())
}

#[tauri::command]
async fn set_provider_key(
    slot: State<'_, SessionSlot>,
    env_name: String,
    value: String,
) -> Result<Vec<InternInfo>, String> {
    let _ = active(&slot).await?;
    stynx_code_app::save_provider_key(&env_name, &value).map_err(|e| e.to_string())?;
    Ok(current_interns())
}

#[tauri::command]
async fn list_sessions(slot: State<'_, SessionSlot>) -> Result<Vec<SessionSummary>, String> {
    let session = active(&slot).await?;
    Ok(session
        .session_repo
        .list()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|summary| SessionSummary {
            id: summary.id,
            title: summary.title,
            updated_at: summary.updated_at,
            message_count: summary.message_count as u32,
        })
        .collect())
}

#[tauri::command]
async fn load_session(slot: State<'_, SessionSlot>, id: String) -> Result<Vec<Turn>, String> {
    let session = active(&slot).await?;
    let loaded = session.session_repo.load(&id).await.ok().flatten();
    let _ = session.session_repo.set_current(&id).await;

    let Some(mut conversation) = loaded else { return Ok(Vec::new()) };
    if conversation.system.is_none() {
        conversation.system = Some(session.system_prompt.clone());
    }
    let turns = conversation.messages.iter().filter_map(message_to_turn).collect();
    *session.current_session_id.lock().unwrap() = Some(id);
    *session.conversation.lock().await = conversation;
    Ok(turns)
}

#[tauri::command]
async fn new_session(slot: State<'_, SessionSlot>) -> Result<(), String> {
    let session = active(&slot).await?;
    *session.current_session_id.lock().unwrap() = None;
    *session.conversation.lock().await = Conversation {
        system: Some(session.system_prompt.clone()),
        messages: Vec::new(),
    };
    Ok(())
}

#[tauri::command]
async fn delete_session(slot: State<'_, SessionSlot>, id: String) -> Result<(), String> {
    let session = active(&slot).await?;
    let _ = session.session_repo.delete(&id).await;
    Ok(())
}

#[tauri::command]
async fn respond_permission(
    slot: State<'_, SessionSlot>,
    id: u64,
    choice: String,
) -> Result<(), String> {
    let session = active(&slot).await?;
    let mapped = match choice.as_str() {
        "allow_always" => PromptChoice::AllowAlways,
        "deny" => PromptChoice::Deny,
        _ => PromptChoice::AllowOnce,
    };
    if let Some(responder) = session.prompt_responders.lock().unwrap().remove(&id) {
        let _ = responder.send(mapped);
    }
    Ok(())
}

#[tauri::command]
async fn respond_ask_user(
    slot: State<'_, SessionSlot>,
    id: u64,
    answer: Option<String>,
) -> Result<(), String> {
    let session = active(&slot).await?;
    if let Some(responder) = session.question_responders.lock().unwrap().remove(&id) {
        let _ = responder.send(answer);
    }
    Ok(())
}

#[tauri::command]
async fn respond_workspace_message(
    slot: State<'_, SessionSlot>,
    id: u64,
    reply: String,
) -> Result<(), String> {
    let session = active(&slot).await?;
    if let Some(responder) = session.workspace_responders.lock().unwrap().remove(&id) {
        let _ = responder.send(reply);
    }
    Ok(())
}

fn message_to_turn(message: &Message) -> Option<Turn> {
    let text: String = message
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    if text.trim().is_empty() {
        return None;
    }
    let role = match message.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };
    Some(Turn { role: role.to_string(), text })
}

fn main() {
    tauri::Builder::default()
        .manage(SessionSlot::default())
        .invoke_handler(tauri::generate_handler![
            init_session,
            get_session_info,
            send_message,
            cancel,
            set_model,
            set_mode,
            set_thinking,
            set_provider_key,
            list_sessions,
            load_session,
            new_session,
            delete_session,
            respond_permission,
            respond_ask_user,
            respond_workspace_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running stynx desktop");
}
