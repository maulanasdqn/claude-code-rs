use async_trait::async_trait;
use claude_rust_errors::AppResult;
use crate::domain::bridge_types::{BridgeError, BridgeMessage, BridgeMethod};

#[async_trait]
pub trait BridgeMessageHandler: Send + Sync {
    async fn handle_message(&self, msg: BridgeMessage) -> AppResult<BridgeMessage>;
}

pub struct DefaultBridgeHandler;

#[async_trait]
impl BridgeMessageHandler for DefaultBridgeHandler {
    async fn handle_message(&self, msg: BridgeMessage) -> AppResult<BridgeMessage> {
        match msg {
            BridgeMessage::Request { id, method, params } => {
                tracing::debug!(?method, "handling bridge request");
                let result = match method {
                    BridgeMethod::Execute => {
                        tracing::info!("execute request received");
                        serde_json::json!({ "status": "accepted", "params": params })
                    }
                    BridgeMethod::Cancel => {
                        tracing::info!("cancel request received");
                        serde_json::json!({ "status": "cancelled" })
                    }
                    BridgeMethod::GetStatus => {
                        serde_json::json!({ "status": "running" })
                    }
                    BridgeMethod::PermissionPrompt => {
                        serde_json::json!({ "approved": true })
                    }
                    BridgeMethod::ToolResult => {
                        serde_json::json!({ "status": "received" })
                    }
                    BridgeMethod::SetContext => {
                        serde_json::json!({ "status": "ok" })
                    }
                };
                Ok(BridgeMessage::Response {
                    id,
                    result: Some(result),
                    error: None,
                })
            }
            BridgeMessage::Notification { method, .. } => {
                tracing::debug!(method, "received notification (no response)");
                Ok(BridgeMessage::Response {
                    id: String::new(),
                    result: None,
                    error: None,
                })
            }
            BridgeMessage::Response { id, .. } => {
                tracing::warn!(id, "received unexpected response message");
                Ok(BridgeMessage::Response {
                    id,
                    result: None,
                    error: Some(BridgeError {
                        code: -32600,
                        message: "unexpected response message".into(),
                    }),
                })
            }
        }
    }
}
