use claude_rust_auth::Credential;
use claude_rust_types::{ContentBlock, Conversation};
use serde_json::{Value, json};

use super::anthropic_provider::BILLING_HEADER_LINE;

pub fn build_request_body(
    credential: &Credential,
    model: &str,
    conversation: &Conversation,
    tools: &[Value],
    thinking: bool,
    max_tokens: u32,
) -> Value {
    let messages: Vec<Value> = conversation
        .messages
        .iter()
        .map(|msg| {
            let content: Vec<Value> = msg
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(json!({
                        "type": "text",
                        "text": text,
                    })),
                    ContentBlock::ToolUse { id, name, input } => Some(json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": input,
                    })),
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
                        Some(v)
                    }
                    ContentBlock::Image { media_type, data } => Some(json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": data
                        }
                    })),
                    // Skip thinking blocks — the API rejects them if sent back
                    ContentBlock::Thinking { .. } => None,
                })
                .collect();

            json!({
                "role": serde_json::to_value(&msg.role).unwrap_or(json!("user")),
                "content": content,
            })
        })
        .collect();

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": messages,
        "stream": true,
    });

    if credential.is_oauth() {
        let mut system_blocks: Vec<Value> = vec![
            json!({"type": "text", "text": BILLING_HEADER_LINE}),
        ];
        if let Some(system) = &conversation.system {
            system_blocks.push(json!({"type": "text", "text": system}));
        }
        body["system"] = json!(system_blocks);
    } else if let Some(system) = &conversation.system {
        body["system"] = json!(system);
    }

    if thinking {
        let budget = 10000u32;
        // API requires max_tokens > budget_tokens
        if max_tokens <= budget {
            body["max_tokens"] = json!(budget + 4096);
        }
        body["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
    }

    if !tools.is_empty() {
        body["tools"] = json!(tools);
    }

    body
}
