use std::sync::atomic::Ordering;

use stynx_code_types::PermissionMode;
use tauri::State;

use crate::application::session::{SessionSlot, current_interns};
use crate::domain::models::InternInfo;
use crate::infrastructure::commands::active;

#[tauri::command]
pub async fn set_model(slot: State<'_, SessionSlot>, model: String) -> Result<String, String> {
    let session = active(&slot).await?;
    session.provider.set_model(&model);
    Ok(session.provider.model_name())
}

#[tauri::command]
pub async fn set_mode(slot: State<'_, SessionSlot>, mode: String) -> Result<(), String> {
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
pub async fn set_thinking(slot: State<'_, SessionSlot>, enabled: bool) -> Result<(), String> {
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
pub async fn set_provider_key(
    slot: State<'_, SessionSlot>,
    env_name: String,
    value: String,
) -> Result<Vec<InternInfo>, String> {
    let _ = active(&slot).await?;
    stynx_code_app::save_provider_key(&env_name, &value).map_err(|error| error.to_string())?;
    Ok(current_interns())
}
