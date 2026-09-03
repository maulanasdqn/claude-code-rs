use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use stynx_code_app::{AppOptions, build_app};
use stynx_code_engine::QueryEngine;
use stynx_code_memory::SessionRepository;
use stynx_code_provider::AnthropicProvider;
use stynx_code_permission::{PromptBridge, PromptChoice, PromptRequest};
use stynx_code_tools::{QuestionBridge, QuestionRequest};
use stynx_code_types::{
    Conversation, ContentBlock, EngineEvent, Message, PermissionMode, Provider, Role,
};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum StynxError {
    #[error("{message}")]
    Init { message: String },
}

#[derive(uniffi::Enum)]
pub enum FfiEvent {
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

impl From<EngineEvent> for FfiEvent {
    fn from(event: EngineEvent) -> Self {
        match event {
            EngineEvent::TextDelta(text) => FfiEvent::TextDelta { text },
            EngineEvent::ThinkingDelta(text) => FfiEvent::ThinkingDelta { text },
            EngineEvent::ToolStart { name, id } => FfiEvent::ToolStart { name, id },
            EngineEvent::ToolInput { json_chunk } => FfiEvent::ToolInput { json_chunk },
            EngineEvent::ToolOutput { name, chunk } => FfiEvent::ToolOutput { name, chunk },
            EngineEvent::ToolResult { name, output, is_error } => {
                FfiEvent::ToolResult { name, output, is_error }
            }
            EngineEvent::Usage { input_tokens, output_tokens } => {
                FfiEvent::Usage { input_tokens, output_tokens }
            }
            EngineEvent::TurnComplete => FfiEvent::TurnComplete,
            EngineEvent::Compacted { original_turns } => {
                FfiEvent::Compacted { original_turns: original_turns as u32 }
            }
            EngineEvent::ModeChanged { mode } => FfiEvent::ModeChanged { mode: mode.to_string() },
            EngineEvent::HookOutput { source, output } => FfiEvent::HookOutput { source, output },
            EngineEvent::SubAgentProgress { label, summary } => {
                FfiEvent::SubAgentProgress { label, summary }
            }
            EngineEvent::SubAgentDone { label } => FfiEvent::SubAgentDone { label },
            EngineEvent::RetryNotice { attempt, max_attempts, delay_ms, message } => {
                FfiEvent::RetryNotice { attempt, max_attempts, delay_ms, message }
            }
            EngineEvent::Error(message) => FfiEvent::Error { message },
        }
    }
}

#[uniffi::export(callback_interface)]
pub trait EventListener: Send + Sync {
    fn on_event(&self, event: FfiEvent);
}

#[derive(uniffi::Record)]
pub struct FfiSessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub message_count: u32,
}

#[derive(uniffi::Record)]
pub struct FfiTurn {
    pub role: String,
    pub text: String,
}

#[derive(uniffi::Record)]
pub struct FfiImage {
    pub media_type: String,
    pub data: String,
}

#[derive(uniffi::Record, Clone)]
pub struct FfiInternInfo {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub description: String,
    pub key_env: String,
    pub available: bool,
}

fn current_interns() -> Vec<FfiInternInfo> {
    stynx_code_app::list_interns_current()
        .into_iter()
        .map(|intern| FfiInternInfo {
            name: intern.name,
            provider: intern.provider,
            model: intern.model,
            description: intern.description,
            key_env: intern.key_env,
            available: intern.available,
        })
        .collect()
}

type SharedListener = Arc<StdMutex<Option<Arc<dyn EventListener>>>>;
type PromptResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<PromptChoice>>>>;
type QuestionResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<Option<String>>>>>;
type WorkspaceResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<String>>>>;

#[derive(uniffi::Object)]
pub struct StynxSession {
    runtime: Arc<Runtime>,
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
    listener: SharedListener,
    prompt_responders: PromptResponders,
    question_responders: QuestionResponders,
    workspace_responders: WorkspaceResponders,
    current_task: Arc<StdMutex<Option<tokio::task::AbortHandle>>>,
}

#[uniffi::export]
impl StynxSession {
    #[uniffi::constructor]
    pub fn new(workspace_path: String) -> Result<Arc<Self>, StynxError> {
        Self::build_session(workspace_path, None)
    }

    #[uniffi::constructor(name = "new_with_provider")]
    pub fn new_with_provider(
        workspace_path: String,
        provider: String,
    ) -> Result<Arc<Self>, StynxError> {
        Self::build_session(workspace_path, Some(provider))
    }

    pub fn model_id(&self) -> String {
        self.provider.model_name()
    }

    pub fn set_model(&self, model: String) {
        self.provider.set_model(&model);
    }

    pub fn claude_available(&self) -> bool {
        self.anthropic.is_some()
    }

    pub fn current_provider(&self) -> String {
        self.provider_label.clone()
    }

    pub fn main_providers(&self) -> Vec<String> {
        stynx_code_app::list_main_providers()
    }

    pub fn claude_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-8".to_string(),
            "claude-sonnet-4-6".to_string(),
            "claude-haiku-4-5-20251001".to_string(),
        ]
    }

    pub fn deepseek_models(&self) -> Vec<String> {
        vec![
            "deepseek-v4-pro".to_string(),
            "deepseek-v4-flash".to_string(),
        ]
    }

    pub fn list_interns(&self) -> Vec<FfiInternInfo> {
        current_interns()
    }

    pub fn set_provider_key(&self, env_name: String, value: String) -> bool {
        stynx_code_app::save_provider_key(&env_name, &value).is_ok()
    }

    pub fn thinking_enabled(&self) -> bool {
        self.anthropic
            .as_ref()
            .map(|provider| provider.thinking_enabled())
            .unwrap_or(false)
    }

    pub fn set_thinking(&self, enabled: bool) {
        let Some(provider) = &self.anthropic else { return };
        if enabled {
            provider.set_max_tokens(16000);
            provider.set_thinking_budget(8000);
        } else if provider.thinking_enabled() {
            provider.toggle_thinking();
        }
    }

    pub fn set_mode(&self, mode: String) {
        let value = match mode.as_str() {
            "auto" | "auto_accept" => PermissionMode::AutoAccept,
            "plan" => PermissionMode::Plan,
            "bypass" => PermissionMode::Bypass,
            _ => PermissionMode::Normal,
        };
        self.mode_flag.store(value as u8, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        if let Some(handle) = self.current_task.lock().unwrap().take() {
            handle.abort();
        }
        if let Some(listener) = self.listener.lock().unwrap().as_ref() {
            listener.on_event(FfiEvent::Idle);
        }
    }

    pub fn send_message(&self, text: String, listener: Box<dyn EventListener>) {
        let content = stynx_code_commands::expand_message_content(&text);
        self.dispatch(Message { role: Role::User, content }, listener);
    }

    pub fn send_message_with_images(
        &self,
        text: String,
        images: Vec<FfiImage>,
        listener: Box<dyn EventListener>,
    ) {
        let mut content: Vec<ContentBlock> = images
            .into_iter()
            .map(|image| ContentBlock::Image { media_type: image.media_type, data: image.data })
            .collect();
        if !text.trim().is_empty() {
            content.extend(stynx_code_commands::expand_message_content(&text));
        }
        self.dispatch(Message { role: Role::User, content }, listener);
    }

    pub fn list_sessions(&self) -> Vec<FfiSessionSummary> {
        let repo = self.session_repo.clone();
        self.runtime
            .block_on(async move { repo.list().await })
            .unwrap_or_default()
            .into_iter()
            .map(|summary| FfiSessionSummary {
                id: summary.id,
                title: summary.title,
                updated_at: summary.updated_at,
                message_count: summary.message_count as u32,
            })
            .collect()
    }

    pub fn load_session(&self, id: String) -> Vec<FfiTurn> {
        let repo = self.session_repo.clone();
        let lookup = id.clone();
        let loaded = self.runtime.block_on(async move {
            let conversation = repo.load(&lookup).await.ok().flatten();
            let _ = repo.set_current(&lookup).await;
            conversation
        });

        let Some(mut conversation) = loaded else { return Vec::new() };
        if conversation.system.is_none() {
            conversation.system = Some(self.system_prompt.clone());
        }
        let turns = conversation.messages.iter().filter_map(message_to_turn).collect();
        *self.current_session_id.lock().unwrap() = Some(id);
        *self.conversation.blocking_lock() = conversation;
        turns
    }

    pub fn new_session(&self) {
        *self.current_session_id.lock().unwrap() = None;
        *self.conversation.blocking_lock() = Conversation {
            system: Some(self.system_prompt.clone()),
            messages: Vec::new(),
        };
    }

    pub fn delete_session(&self, id: String) {
        let repo = self.session_repo.clone();
        let _ = self.runtime.block_on(async move { repo.delete(&id).await });
    }

    pub fn respond_permission(&self, id: u64, choice: String) {
        let mapped = match choice.as_str() {
            "allow_always" => PromptChoice::AllowAlways,
            "deny" => PromptChoice::Deny,
            _ => PromptChoice::AllowOnce,
        };
        if let Some(responder) = self.prompt_responders.lock().unwrap().remove(&id) {
            let _ = responder.send(mapped);
        }
    }

    pub fn respond_ask_user(&self, id: u64, answer: Option<String>) {
        if let Some(responder) = self.question_responders.lock().unwrap().remove(&id) {
            let _ = responder.send(answer);
        }
    }

    pub fn respond_workspace_message(&self, id: u64, reply: String) {
        if let Some(responder) = self.workspace_responders.lock().unwrap().remove(&id) {
            let _ = responder.send(reply);
        }
    }
}

impl StynxSession {
    fn build_session(
        workspace_path: String,
        provider_override: Option<String>,
    ) -> Result<Arc<Self>, StynxError> {
        let runtime = Runtime::new().map_err(|error| StynxError::Init {
            message: format!("failed to start async runtime: {error}"),
        })?;

        let mut options = AppOptions::new(workspace_path.clone());
        options.provider_override = provider_override;
        let handles = runtime
            .block_on(build_app(options))
            .map_err(|message| StynxError::Init { message })?;

        let listener: SharedListener = Arc::new(StdMutex::new(None));
        let prompt_responders: PromptResponders = Arc::new(StdMutex::new(HashMap::new()));
        let question_responders: QuestionResponders = Arc::new(StdMutex::new(HashMap::new()));
        let workspace_responders: WorkspaceResponders = Arc::new(StdMutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU64::new(1));

        let (prompt_bridge, prompt_rx) = PromptBridge::new();
        handles.permission.install_prompt_bridge(prompt_bridge);
        spawn_prompt_drain(
            &runtime,
            prompt_rx,
            listener.clone(),
            prompt_responders.clone(),
            next_id.clone(),
        );

        let (question_bridge, question_rx) = QuestionBridge::new();
        handles.ask_user_bridge.set(question_bridge);
        spawn_question_drain(
            &runtime,
            question_rx,
            listener.clone(),
            question_responders.clone(),
            next_id.clone(),
        );

        let (workspace_bridge, workspace_rx) = stynx_code_app::WorkspaceBridge::new();
        handles.workspace_bridge.set(workspace_bridge);
        spawn_workspace_drain(
            &runtime,
            workspace_rx,
            listener.clone(),
            workspace_responders.clone(),
            next_id,
        );

        let system_prompt = handles.system_prompt;
        let anthropic = handles.anthropic.clone();
        if let Some(provider) = &anthropic {
            provider.set_max_tokens(16000);
            provider.set_thinking_budget(8000);
        }
        let conversation = Conversation {
            system: Some(system_prompt.clone()),
            messages: Vec::new(),
        };

        Ok(Arc::new(Self {
            runtime: Arc::new(runtime),
            engine: handles.engine,
            conversation: Arc::new(Mutex::new(conversation)),
            mode_flag: handles.mode_flag,
            provider: handles.provider,
            provider_label: handles.provider_label,
            system_prompt,
            workspace_path,
            anthropic,
            session_repo: handles.session_repo,
            current_session_id: Arc::new(StdMutex::new(None)),
            listener,
            prompt_responders,
            question_responders,
            workspace_responders,
            current_task: Arc::new(StdMutex::new(None)),
        }))
    }

    fn dispatch(&self, message: Message, listener: Box<dyn EventListener>) {
        let listener: Arc<dyn EventListener> = Arc::from(listener);
        *self.listener.lock().unwrap() = Some(listener.clone());

        let engine = self.engine.clone();
        let conversation = self.conversation.clone();
        let session_repo = self.session_repo.clone();
        let current_session_id = self.current_session_id.clone();
        let workspace_path = self.workspace_path.clone();
        let sink = listener.clone();

        let handle = self.runtime.spawn(async move {
            let mut guard = conversation.lock().await;
            guard.push(message);
            let snapshot = guard.clone();

            let result = engine
                .run(snapshot, move |event| {
                    sink.on_event(FfiEvent::from(event));
                })
                .await;

            match result {
                Ok(updated) => {
                    // A normal run strictly appends to the conversation; a shorter
                    // (or equal) result means mid-run compaction rewrote history.
                    // Archive the full transcript under the old session id and
                    // continue in a fresh session so the original is never lost.
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
                    // Capture into Truncus memory (throttled; no-op if unconfigured).
                    let captured_id = current_session_id.lock().unwrap().clone();
                    stynx_code_truncus::capture(captured_id.as_deref(), &workspace_path, &updated, false).await;
                    *guard = updated;
                    listener.on_event(FfiEvent::Idle);
                }
                Err(error) => {
                    listener.on_event(FfiEvent::Error { message: error.to_string() });
                    listener.on_event(FfiEvent::Idle);
                }
            }
        });

        *self.current_task.lock().unwrap() = Some(handle.abort_handle());
    }
}

fn spawn_workspace_drain(
    runtime: &Runtime,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<stynx_code_app::workspace_bridge::WorkspaceRequest>,
    listener: SharedListener,
    responders: WorkspaceResponders,
    next_id: Arc<AtomicU64>,
) {
    runtime.spawn(async move {
        while let Some(request) = receiver.recv().await {
            let stynx_code_app::workspace_bridge::WorkspaceRequest { target, task, responder } = request;
            let id = next_id.fetch_add(1, Ordering::Relaxed);
            responders.lock().unwrap().insert(id, responder);
            let delivered = emit(&listener, FfiEvent::WorkspaceMessageRequest { id, target, task });
            if !delivered
                && let Some(responder) = responders.lock().unwrap().remove(&id)
            {
                let _ = responder.send("(no UI connected to route the message)".to_string());
            }
        }
    });
}

fn message_to_turn(message: &Message) -> Option<FfiTurn> {
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
    Some(FfiTurn { role: role.to_string(), text })
}

fn emit(listener: &SharedListener, event: FfiEvent) -> bool {
    let guard = listener.lock().unwrap();
    if let Some(active) = guard.as_ref() {
        active.on_event(event);
        true
    } else {
        false
    }
}

fn spawn_prompt_drain(
    runtime: &Runtime,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<PromptRequest>,
    listener: SharedListener,
    responders: PromptResponders,
    next_id: Arc<AtomicU64>,
) {
    runtime.spawn(async move {
        while let Some(request) = receiver.recv().await {
            let PromptRequest { tool_name, description, responder } = request;
            let id = next_id.fetch_add(1, Ordering::Relaxed);
            responders.lock().unwrap().insert(id, responder);
            let delivered = emit(&listener, FfiEvent::PermissionRequest { id, tool_name, description });
            if !delivered
                && let Some(responder) = responders.lock().unwrap().remove(&id)
            {
                let _ = responder.send(PromptChoice::Deny);
            }
        }
    });
}

fn spawn_question_drain(
    runtime: &Runtime,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<QuestionRequest>,
    listener: SharedListener,
    responders: QuestionResponders,
    next_id: Arc<AtomicU64>,
) {
    runtime.spawn(async move {
        while let Some(request) = receiver.recv().await {
            let QuestionRequest { question, responder } = request;
            let id = next_id.fetch_add(1, Ordering::Relaxed);
            responders.lock().unwrap().insert(id, responder);
            let delivered = emit(&listener, FfiEvent::AskUserRequest { id, question });
            if !delivered
                && let Some(responder) = responders.lock().unwrap().remove(&id)
            {
                let _ = responder.send(None);
            }
        }
    });
}
