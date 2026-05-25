use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BridgeMessage {
    Request {
        id: String,
        method: BridgeMethod,
        params: Value,
    },
    Response {
        id: String,
        result: Option<Value>,
        error: Option<BridgeError>,
    },
    Notification {
        method: String,
        params: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeMethod {
    Execute,
    Cancel,
    GetStatus,
    PermissionPrompt,
    ToolResult,
    SetContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeError {
    pub code: i32,
    pub message: String,
}
