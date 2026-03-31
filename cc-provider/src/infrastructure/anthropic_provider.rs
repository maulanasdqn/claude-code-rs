use claude_rust_auth::Credential;
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{Conversation, Provider, StreamEvent};
use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;

use super::sse_parser::{parse_sse_event, parse_sse_lines};
use super::request_builder::build_request_body;

const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4-20250514";
pub(crate) const OAUTH_DEFAULT_MODEL: &str = "claude-sonnet-4-6";
pub(crate) const MAX_TOKENS: u32 = 8192;

pub(crate) const OAUTH_BETA: &str = "claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14";
pub(crate) const BILLING_HEADER: &str = "x-anthropic-billing-header: cc_version=2.1.87.d34; cc_entrypoint=cli; cch=cbde1;";

pub struct AnthropicProvider {
    client: Client,
    credential: Credential,
    model: std::sync::Mutex<String>,
}

impl AnthropicProvider {
    pub fn new(credential: Credential) -> Self {
        let default_model = if credential.is_oauth() {
            OAUTH_DEFAULT_MODEL
        } else {
            DEFAULT_MODEL
        };

        Self {
            client: Client::new(),
            model: std::sync::Mutex::new(
                std::env::var("MODEL").unwrap_or_else(|_| default_model.to_string()),
            ),
            credential,
        }
    }

    pub fn set_model(&self, model: &str) {
        if let Ok(mut m) = self.model.lock() {
            *m = model.to_string();
        }
    }

    pub fn model_name(&self) -> String {
        self.model
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
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
        let model_display = self.model_name();
        let body = build_request_body(&self.credential, &model_display, conversation, tools);

        let request = match &self.credential {
            Credential::ClaudeCodeOAuth { access_token, .. } => {
                let url = format!("{base_url}/v1/messages?beta=true");
                tracing::debug!(model = %model_display, url = %url, "sending OAuth request");

                self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {access_token}"))
                    .header("anthropic-version", "2023-06-01")
                    .header("anthropic-beta", OAUTH_BETA)
                    .header("anthropic-dangerous-direct-browser-access", "true")
                    .header("User-Agent", "claude-cli/2.1.87 (external, cli)")
                    .header("x-app", "cli")
                    .header("content-type", "application/json")
            }
            Credential::ApiKey { api_key, .. } => {
                let url = format!("{base_url}/v1/messages");
                tracing::debug!(model = %model_display, url = %url, "sending API key request");

                self.client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
            }
        };

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
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
            .map(|chunk| {
                let chunk = match chunk {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return vec![StreamEvent::Error {
                            message: e.to_string(),
                        }];
                    }
                };

                let text = String::from_utf8_lossy(&chunk);
                let sse_events = parse_sse_lines(&text);

                sse_events
                    .into_iter()
                    .filter_map(|(event_type, data)| parse_sse_event(&event_type, &data))
                    .collect::<Vec<_>>()
            })
            .flat_map(futures::stream::iter);

        Ok(Box::pin(event_stream))
    }
}
