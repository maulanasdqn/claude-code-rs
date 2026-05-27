use stynx_code_auth::Credential;
use stynx_code_types::{ContentBlock, Conversation, Role};
use serde_json::{Value, json};

use super::anthropic_provider::BILLING_HEADER_LINE;

pub fn build_request_body(
    credential: &Credential,
    model: &str,
    conversation: &Conversation,
    tools: &[Value],
    thinking: bool,
    max_tokens: u32,
    thinking_budget: Option<u32>,
    effort: Option<&str>,
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
                        let boundary = content.char_indices()
                            .map(|(i, _)| i)
                            .take_while(|&i| i < MAX_RESULT)
                            .last()
                            .unwrap_or(0);
                        format!("{}…\n[truncated {} chars]", &content[..boundary], content.len() - boundary)
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

    let messages = sanitize_messages(messages);

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
        let model_lower = model.to_lowercase();
        let supports_adaptive = model_lower.contains("opus-4-6") || model_lower.contains("sonnet-4-6");

        if supports_adaptive {

            body["thinking"] = json!({"type": "adaptive"});
        } else {
            match thinking_budget {
                Some(budget) => {
                    let effective_max = max_tokens.max(budget + 16384);
                    body["max_tokens"] = json!(effective_max);
                    let effective_budget = budget.min(effective_max - 1);
                    body["thinking"] = json!({"type": "enabled", "budget_tokens": effective_budget});
                }
                None => {
                    let budget = 5000u32;
                    if max_tokens <= budget { body["max_tokens"] = json!(budget + 4096); }
                    body["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
                }
            }
        }
    }

    if let Some(eff) = effort {
        body["output_config"] = json!({"effort": eff});
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

#[cfg(test)]
mod tests {
    use super::*;
    use stynx_code_types::{Conversation, Message, ContentBlock, Role};

    fn api_key_credential() -> Credential {
        Credential::ApiKey {
            api_key: "test-key".into(),
            base_url: "https://api.anthropic.com".into(),
        }
    }

    fn simple_conversation() -> Conversation {
        Conversation {
            system: Some("You are helpful.".into()),
            messages: vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: "hello".into() }],
            }],
        }
    }

    #[test]
    fn test_effort_sets_output_config() {
        let body = build_request_body(
            &api_key_credential(), "claude-sonnet-4-6", &simple_conversation(),
            &[], false, 4096, None, Some("high"),
        );
        assert_eq!(body["output_config"]["effort"], "high");
    }

    #[test]
    fn test_effort_max_sets_output_config() {
        let body = build_request_body(
            &api_key_credential(), "claude-opus-4-6", &simple_conversation(),
            &[], true, 64000, None, Some("max"),
        );
        assert_eq!(body["output_config"]["effort"], "max");

        assert_eq!(body["thinking"]["type"], "adaptive");
    }

    #[test]
    fn test_no_effort_omits_output_config() {
        let body = build_request_body(
            &api_key_credential(), "claude-sonnet-4-6", &simple_conversation(),
            &[], false, 4096, None, None,
        );
        assert!(body.get("output_config").is_none());
    }

    #[test]
    fn test_adaptive_thinking_for_opus() {
        let body = build_request_body(
            &api_key_credential(), "claude-opus-4-6", &simple_conversation(),
            &[], true, 16000, None, None,
        );
        assert_eq!(body["thinking"]["type"], "adaptive");
        assert!(body["thinking"].get("budget_tokens").is_none());
    }

    #[test]
    fn test_adaptive_thinking_for_sonnet_4_6() {
        let body = build_request_body(
            &api_key_credential(), "claude-sonnet-4-6", &simple_conversation(),
            &[], true, 16000, None, None,
        );
        assert_eq!(body["thinking"]["type"], "adaptive");
    }

    #[test]
    fn test_budget_thinking_for_older_model() {
        let body = build_request_body(
            &api_key_credential(), "claude-haiku-4-5-20251001", &simple_conversation(),
            &[], true, 16000, None, None,
        );
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["thinking"]["budget_tokens"], 5000);
    }

    #[test]
    fn test_custom_thinking_budget() {
        let body = build_request_body(
            &api_key_credential(), "claude-haiku-4-5-20251001", &simple_conversation(),
            &[], true, 4096, Some(10000), None,
        );
        assert_eq!(body["thinking"]["type"], "enabled");

        assert_eq!(body["thinking"]["budget_tokens"], 10000);
        assert_eq!(body["max_tokens"], 26384);
    }

    #[test]
    fn test_thinking_disabled_no_thinking_block() {
        let body = build_request_body(
            &api_key_credential(), "claude-opus-4-6", &simple_conversation(),
            &[], false, 4096, None, None,
        );
        assert!(body.get("thinking").is_none());
    }
}

fn sanitize_messages(mut messages: Vec<Value>) -> Vec<Value> {
    let mut i = 0;
    while i < messages.len() {
        if messages[i].get("role").and_then(|r| r.as_str()) != Some("assistant") {
            i += 1;
            continue;
        }
        let tool_use_ids: Vec<String> = messages[i]["content"]
            .as_array()
            .map(|arr| arr.iter()
                .filter_map(|b| if b["type"] == "tool_use" {
                    b["id"].as_str().map(String::from)
                } else { None })
                .collect())
            .unwrap_or_default();
        if tool_use_ids.is_empty() { i += 1; continue; }

        let next_result_ids: Vec<String> = messages.get(i + 1)
            .and_then(|m| m["content"].as_array())
            .map(|arr| arr.iter()
                .filter_map(|b| if b["type"] == "tool_result" {
                    b["tool_use_id"].as_str().map(String::from)
                } else { None })
                .collect())
            .unwrap_or_default();

        let orphaned: Vec<&str> = tool_use_ids.iter()
            .filter(|id| !next_result_ids.contains(id))
            .map(String::as_str)
            .collect();

        if orphaned.is_empty() { i += 1; continue; }

        if let Some(arr) = messages[i]["content"].as_array_mut() {
            arr.retain(|b| !(b["type"] == "tool_use"
                && b["id"].as_str().map(|id| orphaned.contains(&id)).unwrap_or(false)));
        }

        if messages[i]["content"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
            messages.remove(i);
        } else {
            i += 1;
        }
    }
    messages
}
