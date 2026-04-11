use async_trait::async_trait;
use claude_rust_errors::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub name: String,
    pub properties: HashMap<String, String>,
    pub timestamp: u64,
}

#[async_trait]
pub trait AnalyticsService: Send + Sync {
    async fn track_event(&self, name: &str, properties: HashMap<String, String>) -> AppResult<()>;
    async fn flush(&self) -> AppResult<()>;
}

pub struct LocalAnalytics {
    path: PathBuf,
}

impl LocalAnalytics {
    pub fn new() -> Self {
        let path = home_claude_dir().join("analytics.jsonl");
        Self { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }
}

fn home_claude_dir() -> PathBuf {
    claude_rust_config::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[async_trait]
impl AnalyticsService for LocalAnalytics {
    async fn track_event(&self, name: &str, properties: HashMap<String, String>) -> AppResult<()> {
        let event = AnalyticsEvent {
            name: name.to_string(),
            properties,
            timestamp: now_secs(),
        };

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| -> claude_rust_errors::AppError {
                    anyhow::anyhow!("failed to create analytics dir: {e}").into()
                })?;
        }

        let mut line = serde_json::to_string(&event)?;
        line.push('\n');

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|e| -> claude_rust_errors::AppError {
                anyhow::anyhow!("failed to open analytics file: {e}").into()
            })?;

        file.write_all(line.as_bytes())
            .await
            .map_err(|e| -> claude_rust_errors::AppError {
                anyhow::anyhow!("failed to write analytics event: {e}").into()
            })?;

        tracing::info!(event_name = name, "tracked analytics event");
        Ok(())
    }

    async fn flush(&self) -> AppResult<()> {
        // Local file analytics flushes on each write; nothing to do here.
        Ok(())
    }
}
