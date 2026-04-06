use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

use claude_rust_auth::Credential;
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{Conversation, PermissionMode, Provider, StreamEvent};
use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;

use super::sse_parser::{parse_sse_block, parse_sse_event};
use super::request_builder::build_request_body;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RateLimit {
    pub utilization: Option<f64>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtraUsage {
    pub is_enabled: bool,
    pub monthly_limit: Option<f64>,
    pub used_credits: Option<f64>,
    pub utilization: Option<f64>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Utilization {
    pub five_hour: Option<RateLimit>,
    pub seven_day: Option<RateLimit>,
    pub seven_day_opus: Option<RateLimit>,
    pub seven_day_sonnet: Option<RateLimit>,
    pub extra_usage: Option<ExtraUsage>,
}

const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4-20250514";
pub(crate) const OAUTH_DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const OPUS_MODEL: &str = "claude-opus-4-6";
pub(crate) const MAX_TOKENS: u32 = 4096;

pub(crate) const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20,interleaved-thinking-2025-05-14,claude-code-20250219,prompt-caching-2024-07-31";
pub(crate) const BILLING_HEADER_LINE: &str = "x-anthropic-billing-header: cc_version=2.1.87.d34; cc_entrypoint=cli;";

pub struct AnthropicProvider {
    client: Client,
    credential: Credential,
    model: std::sync::Mutex<String>,
    mode: Arc<AtomicU8>,
    thinking: Arc<AtomicBool>,
    max_tokens: AtomicU32,
}

impl AnthropicProvider {
    pub fn new(credential: Credential, mode: Arc<AtomicU8>) -> Self {
        let default_model = if credential.is_oauth() {
            OAUTH_DEFAULT_MODEL
        } else {
            DEFAULT_MODEL
        };

        Self {
            client: Client::new(),
            model: std::sync::Mutex::new(default_model.to_string()),
            credential,
            mode,
            thinking: Arc::new(AtomicBool::new(false)), // off by default; enable with /think
            max_tokens: AtomicU32::new(MAX_TOKENS),
        }
    }

    pub fn set_model(&self, model: &str) {
        if let Ok(mut m) = self.model.lock() {
            *m = model.to_string();
        }
    }

    pub fn set_max_tokens(&self, n: u32) {
        self.max_tokens.store(n, Ordering::Relaxed);
    }

    pub fn model_name(&self) -> String {
        self.effective_model()
    }

    pub fn toggle_thinking(&self) -> bool {
        let current = self.thinking.load(Ordering::Relaxed);
        let next = !current;
        self.thinking.store(next, Ordering::Relaxed);
        next
    }

    pub fn thinking_enabled(&self) -> bool {
        self.thinking.load(Ordering::Relaxed)
    }

    pub fn is_oauth(&self) -> bool {
        self.credential.is_oauth()
    }

    pub async fn fetch_usage(&self) -> AppResult<Utilization> {
        let access_token = match &self.credential {
            Credential::ClaudeCodeOAuth { access_token, .. } => access_token.clone(),
            _ => {
                return Err(AppError::Provider(
                    "/usage is only available for Claude AI subscribers (OAuth login)".to_string(),
                ));
            }
        };

        let url = format!("{}/api/oauth/usage", self.credential.base_url());
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("anthropic-dangerous-direct-browser-access", "true")
            .header("User-Agent", "claude-cli/2.1.87 (external, cli)")
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("usage request failed: {e}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Provider(format!("usage API error: {body}")));
        }

        response
            .json::<Utilization>()
            .await
            .map_err(|e| AppError::Provider(format!("failed to parse usage response: {e}")))
    }

    fn effective_model(&self) -> String {
        let stored = self.model.lock().map(|m| m.clone()).unwrap_or_default();
        match stored.as_str() {
            "opusplan" => {
                if PermissionMode::load(&self.mode) == PermissionMode::Plan {
                    OPUS_MODEL.to_string()
                } else if self.credential.is_oauth() {
                    OAUTH_DEFAULT_MODEL.to_string()
                } else {
                    DEFAULT_MODEL.to_string()
                }
            }
            _ => stored,
        }
    }
}

#[async_trait::async_trait]
impl Provider for AnthropicProvider {
    async fn stream(
        &self,
        conversation: &Conversation,
        tools: &[Value],
    ) -> AppResult<BoxStream<'static, StreamEvent>> {
        let base_url = self.credential.base_url();
        let model_display = self.effective_model();
        let thinking = self.thinking.load(Ordering::Relaxed);
        let max_tokens = self.max_tokens.load(Ordering::Relaxed);
        let body = build_request_body(&self.credential, &model_display, conversation, tools, thinking, max_tokens);

        let request = match &self.credential {
            Credential::ClaudeCodeOAuth { access_token, .. } | Credential::AuthToken { token: access_token, .. } => {
                let url = format!("{base_url}/v1/messages?beta=true");
                tracing::debug!(model = %model_display, url = %url, "sending OAuth request");

                self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {access_token}"))
                    .header("anthropic-version", "2023-06-01")
                    .header("anthropic-beta", OAUTH_BETA_HEADER)
                    .header("anthropic-dangerous-direct-browser-access", "true")
                    .header("User-Agent", "claude-cli/2.1.87 (external, cli)")
                    .header("x-app", "cli")
                    .header("content-type", "application/json")
            }
            Credential::ApiKey { api_key, .. } => {
                let url = format!("{base_url}/v1/messages");
                tracing::debug!(model = %model_display, url = %url, "sending API key request");

                let mut rb = self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json");

                let beta = if thinking {
                    "prompt-caching-2024-07-31,interleaved-thinking-2025-05-14"
                } else {
                    "prompt-caching-2024-07-31"
                };
                rb = rb.header("anthropic-beta", beta);
                rb
            }
        };

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read body".into());
            return Err(AppError::Provider(format!(
                "API returned {status}: {body}"
            )));
        }

        let byte_stream = response.bytes_stream();

        let event_stream = byte_stream
            .scan(String::new(), |buf, chunk| {
                let events: Vec<StreamEvent> = match chunk {
                    Err(e) => vec![StreamEvent::Error { message: e.to_string() }],
                    Ok(bytes) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                        let mut events = Vec::new();
                        while let Some(pos) = buf.find("\n\n") {
                            let block = buf[..pos].to_string();
                            *buf = buf[pos + 2..].to_string();
                            if let Some((et, d)) = parse_sse_block(&block) {
                                events.extend(parse_sse_event(&et, &d));
                            }
                        }
                        events
                    }
                };
                async move { Some(events) }
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(event_stream))
    }
}
