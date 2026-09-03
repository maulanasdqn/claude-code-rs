use stynx_code_permission::PromptChoice;
use stynx_code_types::{ContentBlock, Message, Role};
use tauri::{AppHandle, State};

use crate::application::dispatch::dispatch;
use crate::application::session::SessionSlot;
use crate::domain::models::ImagePayload;
use crate::infrastructure::commands::active;
use crate::infrastructure::emitter::emit;

#[tauri::command]
pub async fn send_message(
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
    let sink_app = app.clone();
    dispatch(session, Message { role: Role::User, content }, move |event| {
        let _ = emit(&sink_app, event);
    });
    Ok(())
}

#[tauri::command]
pub async fn cancel(app: AppHandle, slot: State<'_, SessionSlot>) -> Result<(), String> {
    let session = active(&slot).await?;
    if let Some(handle) = session.current_task.lock().unwrap().take() {
        handle.abort();
    }
    let _ = emit(&app, crate::domain::ui_event::UiEvent::Idle);
    Ok(())
}

#[tauri::command]
pub async fn respond_permission(
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
pub async fn respond_ask_user(
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
pub async fn respond_workspace_message(
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
