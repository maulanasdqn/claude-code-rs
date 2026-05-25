use std::io::{self, BufRead, Write};
use stynx_code_errors::AppResult;
use crate::domain::bridge_types::BridgeMessage;
use crate::application::bridge_messaging::BridgeMessageHandler;

pub struct StdioTransport;

impl StdioTransport {
    pub fn new() -> Self {
        Self
    }

    pub async fn run<H: BridgeMessageHandler>(&self, handler: H) -> AppResult<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line.map_err(|e| stynx_code_errors::AppError::Internal(anyhow::anyhow!(e)))?;
            if line.trim().is_empty() {
                continue;
            }

            let msg: BridgeMessage = match serde_json::from_str(&line) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(?e, "failed to parse message");
                    continue;
                }
            };

            let response = handler.handle_message(msg).await?;
            let mut out = stdout.lock();
            let serialized = serde_json::to_string(&response)
                .map_err(|e| stynx_code_errors::AppError::Internal(anyhow::anyhow!(e)))?;
            writeln!(out, "{}", serialized)
                .map_err(|e| stynx_code_errors::AppError::Internal(anyhow::anyhow!(e)))?;
        }

        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}
