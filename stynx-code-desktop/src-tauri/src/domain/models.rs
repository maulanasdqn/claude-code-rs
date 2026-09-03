use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub model_id: String,
    pub current_provider: String,
    pub main_providers: Vec<String>,
    pub claude_available: bool,
    pub claude_models: Vec<String>,
    pub deepseek_models: Vec<String>,
    pub interns: Vec<InternInfo>,
    pub thinking_enabled: bool,
    pub workspace_path: String,
    pub project_name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InternInfo {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub description: String,
    pub key_env: String,
    pub available: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub message_count: u32,
}

#[derive(Serialize)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePayload {
    pub media_type: String,
    pub data: String,
}
