use claude_rust_auth::Credential;
use claude_rust_types::{ContentBlock, Conversation, Role};
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
    let total = conversation.messages.len();
    let messages: Vec<Value> = conversation.messages.iter().enumerate().map(|(mi, msg)| {
        let last_mi = total.saturating_sub(2);
        let last_bi = msg.content.len().saturating_sub(1);
        let content: Vec<Value> = msg.content.iter().enumerate().filter_map(|(bi, block)| {
            let cache = matches!(msg.role, Role::User) && mi == last_mi && bi == last_bi && total >= 3;
            match block {
                ContentBlock::Text { text } => {
                    let mut v = json!({"type": "text", "text": text});
                    if cache { v["cache_control"] = json!({"type": "ephemeral"}); }
                    Some(v)
                }
                ContentBlock::ToolUse { id, name, input } => Some(json!({
                    "type": "tool_use", "id": id, "name": name, "input": input,
                })),
                ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                    const MAX_RESULT: usize = 8_000;
                    let content = if content.len() > MAX_RESULT {
                        format!("{}…\n[truncated {} chars]", &content[..MAX_RESULT], content.len() - MAX_RESULT)
                    } else {
                        content.clone()
                    };
                    let mut v = json!({
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": content,
                    });
                    if let Some(true) = is_error { v["is_error"] = json!(true); }
                    if cache { v["cache_control"] = json!({"type": "ephemeral"}); }
                    Some(v)
                }
                ContentBlock::Image { media_type, data } => Some(json!({
                    "type": "image",
                    "source": {"type": "base64", "media_type": media_type, "data": data}
                })),
                ContentBlock::Thinking { .. } => None,
            }
        }).collect();
        json!({"role": serde_json::to_value(&msg.role).unwrap_or(json!("user")), "content": content})
    }).collect();

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": messages,
        "stream": true,
    });

    if credential.is_oauth() {
        let mut sys: Vec<Value> = vec![json!({"type": "text", "text": BILLING_HEADER_LINE})];
        if let Some(s) = &conversation.system {
            sys.push(json!({"type": "text", "text": s}));
        }
        if let Some(last) = sys.last_mut() {
            last["cache_control"] = json!({"type": "ephemeral"});
        }
        body["system"] = json!(sys);
    } else if let Some(s) = &conversation.system {
        body["system"] = json!([{"type": "text", "text": s, "cache_control": {"type": "ephemeral"}}]);
    }

    if thinking {
        let budget = 5000u32;
        if max_tokens <= budget { body["max_tokens"] = json!(budget + 4096); }
        body["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
    }

    if !tools.is_empty() {
        let mut tools_arr = tools.to_vec();
        if let Some(last) = tools_arr.last_mut() {
            last["cache_control"] = json!({"type": "ephemeral"});
        }
        body["tools"] = json!(tools_arr);
    }

    body
}
