use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub permissions: PermissionSettings,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSettings {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}
