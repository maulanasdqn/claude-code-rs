use stynx_code_types::Conversation;
use tauri::State;

use crate::application::session::{SessionSlot, message_to_turn};
use crate::domain::models::{SessionSummary, Turn};
use crate::infrastructure::commands::active;

#[tauri::command]
pub async fn list_sessions(slot: State<'_, SessionSlot>) -> Result<Vec<SessionSummary>, String> {
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
pub async fn load_session(slot: State<'_, SessionSlot>, id: String) -> Result<Vec<Turn>, String> {
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
pub async fn new_session(slot: State<'_, SessionSlot>) -> Result<(), String> {
    let session = active(&slot).await?;
    *session.current_session_id.lock().unwrap() = None;
    *session.conversation.lock().await = Conversation {
        system: Some(session.system_prompt.clone()),
        messages: Vec::new(),
    };
    Ok(())
}

#[tauri::command]
pub async fn delete_session(slot: State<'_, SessionSlot>, id: String) -> Result<(), String> {
    let session = active(&slot).await?;
    let _ = session.session_repo.delete(&id).await;
    Ok(())
}
