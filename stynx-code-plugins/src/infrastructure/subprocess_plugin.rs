use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};

use stynx_code_errors::{AppError, AppResult};

pub struct SubprocessPlugin {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: BufReader<ChildStdout>,
}

impl SubprocessPlugin {
    pub fn new(mut child: Child) -> AppResult<Self> {
        let stdin = child.stdin.take().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("Failed to capture plugin stdin"))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!("Failed to capture plugin stdout"))
        })?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> AppResult<Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let mut line = serde_json::to_string(&request)?;
        line.push('\n');

        self.stdin.write_all(line.as_bytes()).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to write to plugin stdin: {e}"))
        })?;
        self.stdin.flush().await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to flush plugin stdin: {e}"))
        })?;

        let mut response_line = String::new();
        self.stdout
            .read_line(&mut response_line)
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to read plugin response: {e}"))
            })?;

        if response_line.is_empty() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Plugin closed stdout unexpectedly"
            )));
        }

        let response: Value = serde_json::from_str(response_line.trim()).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Invalid JSON response from plugin: {e}"))
        })?;

        if let Some(error) = response.get("error") {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Plugin returned error: {error}"
            )));
        }

        Ok(response["result"].clone())
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}
