
use std::sync::Mutex;

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{
    ContentBlock, Conversation, Message, Provider, Role, StopReason, StreamEvent, UsageStats,
};
use futures::StreamExt;
use futures::stream::BoxStream;
use reqwest::Client;
use serde_json::{Value, json};

pub struct OpenAiProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: Mutex<String>,
    label: String,
    max_tokens: u32,
}

impl OpenAiProvider {
    pub fn new(
        label: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .read_timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            model: Mutex::new(model.into()),
            label: label.into(),
            max_tokens: 4096,
        }
    }

    pub fn deepseek(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new("deepseek", "https://api.deepseek.com/v1", api_key, model)
    }

    pub fn set_model(&self, model: &str) {
        if let Ok(mut m) = self.model.lock() {
            *m = model.to_string();
        }
    }

    pub fn model_name(&self) -> String {
        self.model.lock().map(|m| m.clone()).unwrap_or_default()
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

fn sanitize_schema(schema: Value) -> Value {
    match schema {
        Value::Object(mut map) => {

            for key in [
                "$schema",
                "$id",
                "$ref",
                "title",
                "examples",
                "default",
            ] {
                map.remove(key);
            }

            if let Some(props) = map.get_mut("properties").and_then(Value::as_object_mut) {
                let keys: Vec<String> = props.keys().cloned().collect();
                for k in keys {
                    if let Some(v) = props.remove(&k) {
                        props.insert(k, sanitize_schema(v));
                    }
                }
            }
            if let Some(items) = map.remove("items") {
                map.insert("items".into(), sanitize_schema(items));
            }
            Value::Object(map)
        }
        other => other,
    }
}

fn translate_tools(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .filter_map(|t| {
            let name = t.get("name").and_then(Value::as_str)?;
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            let parameters = t
                .get("input_schema")
                .cloned()
                .map(sanitize_schema)
                .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
            Some(json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": parameters,
                }
            }))
        })
        .collect()
}

fn translate_messages(conv: &Conversation) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    if let Some(sys) = &conv.system {
        if !sys.is_empty() {
            out.push(json!({ "role": "system", "content": sys }));
        }
    }
    for msg in &conv.messages {
        match msg.role {
            Role::User => {
                push_user(msg, &mut out);
            }
            Role::Assistant => {
                push_assistant(msg, &mut out);
            }
        }
    }
    out
}

fn push_user(msg: &Message, out: &mut Vec<Value>) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_results: Vec<(String, String, bool)> = Vec::new();
    for block in &msg.content {
        match block {
            ContentBlock::Text { text } => text_parts.push(text.clone()),
            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                tool_results.push((tool_use_id.clone(), content.clone(), is_error.unwrap_or(false)));
            }
            _ => {}
        }
    }
    for (id, content, is_error) in &tool_results {
        let body = if *is_error {
            format!("[error] {content}")
        } else {
            content.clone()
        };
        out.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "content": body,
        }));
    }
    if !text_parts.is_empty() {
        out.push(json!({
            "role": "user",
            "content": text_parts.join("\n"),
        }));
    }
}

fn push_assistant(msg: &Message, out: &mut Vec<Value>) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();
    for block in &msg.content {
        match block {
            ContentBlock::Text { text } => text_parts.push(text.clone()),
            ContentBlock::Thinking { thinking } => reasoning_parts.push(thinking.clone()),
            ContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(json!({
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": serde_json::to_string(input).unwrap_or_else(|_| "{}".into()),
                    }
                }));
            }
            ContentBlock::ToolResult { .. } | ContentBlock::Image { .. } => {}
        }
    }
    let content = if text_parts.is_empty() {
        Value::Null
    } else {
        Value::String(text_parts.join("\n"))
    };
    let mut obj = serde_json::Map::new();
    obj.insert("role".into(), json!("assistant"));
    obj.insert("content".into(), content);

    if !reasoning_parts.is_empty() {
        obj.insert("reasoning_content".into(), Value::String(reasoning_parts.join("\n")));
    }
    if !tool_calls.is_empty() {
        obj.insert("tool_calls".into(), Value::Array(tool_calls));
    }
    out.push(Value::Object(obj));
}

#[derive(Default)]
struct StreamingToolCall {
    id: String,
    name: String,
    started: bool,
}

fn handle_chunk(
    chunk: &Value,
    tool_calls: &mut Vec<StreamingToolCall>,
    out: &mut Vec<StreamEvent>,
) {
    let Some(choice) = chunk.get("choices").and_then(|c| c.as_array()).and_then(|a| a.first()) else {
        if let Some(usage) = chunk.get("usage") {
            let input = usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0);
            let output = usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0);
            if input > 0 || output > 0 {
                out.push(StreamEvent::Usage {
                    stats: UsageStats { input_tokens: input, output_tokens: output },
                });
            }
        }
        return;
    };

    if let Some(delta) = choice.get("delta") {
        if let Some(text) = delta.get("reasoning_content").and_then(Value::as_str) {
            if !text.is_empty() {
                out.push(StreamEvent::ThinkingDelta { text: text.to_string() });
            }
        }
        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            if !text.is_empty() {
                out.push(StreamEvent::ContentDelta { text: text.to_string() });
            }
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let idx = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                while tool_calls.len() <= idx {
                    tool_calls.push(StreamingToolCall::default());
                }
                let slot = &mut tool_calls[idx];
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    if !id.is_empty() {
                        slot.id = id.to_string();
                    }
                }
                if let Some(func) = call.get("function") {
                    if let Some(name) = func.get("name").and_then(Value::as_str) {
                        if !name.is_empty() {
                            slot.name = name.to_string();
                        }
                    }
                    if !slot.started && !slot.id.is_empty() && !slot.name.is_empty() {
                        slot.started = true;
                        out.push(StreamEvent::ToolUseStart {
                            id: slot.id.clone(),
                            name: slot.name.clone(),
                        });
                    }
                    if let Some(args) = func.get("arguments").and_then(Value::as_str) {
                        if !args.is_empty() {
                            out.push(StreamEvent::ToolUseDelta { json_chunk: args.to_string() });
                        }
                    }
                }
            }
        }
    }

    if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
        let stop = match reason {
            "tool_calls" | "function_call" => StopReason::ToolUse,
            "length" => StopReason::MaxTokens,
            _ => StopReason::EndTurn,
        };
        out.push(StreamEvent::Stop { reason: stop });
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiProvider {
    async fn stream(
        &self,
        conversation: &Conversation,
        tools: &[Value],
    ) -> AppResult<BoxStream<'static, StreamEvent>> {
        let model = self.model_name();
        let messages = translate_messages(conversation);
        let translated_tools = translate_tools(tools);

        let mut body = json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "stream_options": { "include_usage": true },
            "max_tokens": self.max_tokens,
        });
        if !translated_tools.is_empty() {
            body["tools"] = Value::Array(translated_tools);
        }

        let url = format!("{}/chat/completions", self.base_url);
        tracing::debug!(provider = %self.label, model = %model, url = %url, "sending OpenAI-compat request");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("{} request failed: {e}", self.label)))?;

        let status = response.status();
        if !status.is_success() {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let text = response.text().await.unwrap_or_else(|_| "failed to read body".into());
            let prefix = if status.as_u16() == 429 {
                let ms = retry_after.as_deref()
                    .and_then(|v| v.parse::<f64>().ok())
                    .map(|s| (s * 1000.0) as u64)
                    .unwrap_or(60_000);
                format!("[retry_after_ms={ms}] ")
            } else {
                String::new()
            };
            return Err(AppError::Provider(format!(
                "{prefix}{} returned {status}: {text}",
                self.label
            )));
        }

        let byte_stream = response.bytes_stream();
        let event_stream = byte_stream
            .scan(
                (String::new(), Vec::<StreamingToolCall>::new()),
                |state, chunk| {
                    let (buf, tool_calls) = state;
                    let events: Vec<StreamEvent> = match chunk {
                        Err(e) => vec![StreamEvent::Error { message: e.to_string() }],
                        Ok(bytes) => {
                            buf.push_str(&String::from_utf8_lossy(&bytes));
                            let mut events = Vec::new();
                            while let Some(pos) = buf.find('\n') {
                                let line = buf[..pos].trim_end_matches('\r').to_string();
                                *buf = buf[pos + 1..].to_string();
                                let Some(data) = line.strip_prefix("data: ") else {
                                    continue;
                                };
                                let data = data.trim();
                                if data == "[DONE]" {
                                    continue;
                                }
                                let Ok(parsed) = serde_json::from_str::<Value>(data) else {
                                    continue;
                                };
                                handle_chunk(&parsed, tool_calls, &mut events);
                            }
                            events
                        }
                    };
                    async move { Some(events) }
                },
            )
            .flat_map(futures::stream::iter);

        Ok(Box::pin(event_stream))
    }
}
