use cc_auth::Credential;
use cc_types::{ContentBlock, Conversation};
use serde_json::{Value, json};

use super::anthropic_provider::{BILLING_HEADER, MAX_TOKENS};

pub fn build_request_body(
    credential: &Credential,
    model: &str,
    conversation: &Conversation,
    tools: &[Value],
) -> Value {
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
        "model": model,
        "max_tokens": MAX_TOKENS,
        "messages": messages,
        "stream": true,
    });

    if credential.is_oauth() {
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
