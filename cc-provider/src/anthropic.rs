use cc_auth::Credential;
use cc_errors::{AppError, AppResult};
use cc_types::{ContentBlock, Conversation, Provider, StreamEvent};
use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{Value, json};

use crate::stream::{parse_sse_event, parse_sse_lines};

const DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4-20250514";
const OAUTH_DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const MAX_TOKENS: u32 = 8192;

const OAUTH_BETA: &str = "claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14";
const BILLING_HEADER: &str = "x-anthropic-billing-header: cc_version=2.1.87.d34; cc_entrypoint=cli; cch=cbde1;";

pub struct AnthropicProvider {
    client: Client,
    credential: Credential,
    model: String,
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
            model: std::env::var("MODEL").unwrap_or_else(|_| default_model.to_string()),
            credential,
        }
    }

    fn build_request_body(&self, conversation: &Conversation, tools: &[Value]) -> Value {
        let messages: Vec<Value> = conversation
            .messages
            .iter()
            .map(|msg| {
                let content: Vec<Value> = msg
                    .content
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text { text } => json!({
                            "type": "text",
                            "text": text,
                        }),
                        ContentBlock::ToolUse { id, name, input } => json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input,
                        }),
                        ContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            is_error,
                        } => {
                            let mut v = json!({
                                "type": "tool_result",
                                "tool_use_id": tool_use_id,
                                "content": content,
                            });
                            if let Some(true) = is_error {
                                v["is_error"] = json!(true);
                            }
                            v
                        }
                    })
                    .collect();

                json!({
                    "role": serde_json::to_value(&msg.role).unwrap_or(json!("user")),
                    "content": content,
                })
            })
            .collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "messages": messages,
            "stream": true,
        });

        if self.credential.is_oauth() {
            let mut system_blocks = vec![json!({
                "type": "text",
                "text": BILLING_HEADER,
            })];
            if let Some(system) = &conversation.system {
                system_blocks.push(json!({
                    "type": "text",
                    "text": system,
                }));
            }
            body["system"] = json!(system_blocks);
            body["thinking"] = json!({"type": "adaptive"});
        } else if let Some(system) = &conversation.system {
            body["system"] = json!(system);
        }

        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }

        body
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
        let body = self.build_request_body(conversation, tools);

        let request = match &self.credential {
            Credential::ClaudeCodeOAuth { access_token, .. } => {
                let url = format!("{base_url}/v1/messages?beta=true");
                tracing::debug!(model = %self.model, url = %url, "sending OAuth request");

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
                tracing::debug!(model = %self.model, url = %url, "sending API key request");

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
