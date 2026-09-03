use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Mutex as StdMutex};

use stynx_code_engine::QueryEngine;
use stynx_code_memory::SessionRepository;
use stynx_code_permission::PromptChoice;
use stynx_code_provider::AnthropicProvider;
use stynx_code_types::{ContentBlock, Conversation, Message, Provider, Role};
use tokio::sync::{Mutex, oneshot};

use crate::domain::models::{InternInfo, SessionInfo, Turn};

pub type PromptResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<PromptChoice>>>>;
pub type QuestionResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<Option<String>>>>>;
pub type WorkspaceResponders = Arc<StdMutex<HashMap<u64, oneshot::Sender<String>>>>;

pub struct Session {
    pub engine: Arc<QueryEngine>,
    pub conversation: Arc<Mutex<Conversation>>,
    pub mode_flag: Arc<AtomicU8>,
    pub provider: Arc<dyn Provider>,
    pub provider_label: String,
    pub system_prompt: String,
    pub workspace_path: String,
    pub anthropic: Option<Arc<AnthropicProvider>>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub current_session_id: Arc<StdMutex<Option<String>>>,
    pub prompt_responders: PromptResponders,
    pub question_responders: QuestionResponders,
    pub workspace_responders: WorkspaceResponders,
    pub current_task: Arc<StdMutex<Option<tokio::task::AbortHandle>>>,
}

#[derive(Default)]
pub struct SessionSlot(pub Mutex<Option<Arc<Session>>>);

pub fn session_info(session: &Session) -> SessionInfo {
    let project_name = std::path::Path::new(&session.workspace_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
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

pub fn current_interns() -> Vec<InternInfo> {
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

pub fn message_to_turn(message: &Message) -> Option<Turn> {
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
