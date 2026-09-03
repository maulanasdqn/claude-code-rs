use std::sync::{Arc, Mutex as StdMutex};

use stynx_code_app::{AppOptions, build_app};
use stynx_code_types::Conversation;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::application::session::{Session, SessionSlot, session_info};
use crate::domain::models::SessionInfo;
use crate::infrastructure::bridges;
use crate::infrastructure::commands::active;

#[tauri::command]
pub async fn init_session(
    app: AppHandle,
    slot: State<'_, SessionSlot>,
    workspace_path: Option<String>,
    provider: Option<String>,
) -> Result<SessionInfo, String> {
    let path = match workspace_path {
        Some(p) if !p.trim().is_empty() => p,
        _ => std::env::current_dir()
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .to_string(),
    };

    let mut options = AppOptions::new(path.clone());
    options.provider_override = provider;
    let handles = build_app(options).await?;

    let responders = bridges::install(&app, &handles);

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
        workspace_path: path,
        anthropic,
        session_repo: handles.session_repo,
        current_session_id: Arc::new(StdMutex::new(None)),
        prompt_responders: responders.prompt,
        question_responders: responders.question,
        workspace_responders: responders.workspace,
        current_task: Arc::new(StdMutex::new(None)),
    });

    let info = session_info(&session);
    *slot.0.lock().await = Some(session);
    Ok(info)
}

#[tauri::command]
pub async fn get_session_info(slot: State<'_, SessionSlot>) -> Result<SessionInfo, String> {
    Ok(session_info(active(&slot).await?.as_ref()))
}
