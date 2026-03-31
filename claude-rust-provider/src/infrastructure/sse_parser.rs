use claude_rust_types::{StopReason, StreamEvent, UsageStats};
use serde_json::Value;

pub fn parse_sse_event(event_type: &str, data: &str) -> Vec<StreamEvent> {
    let v: Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    match event_type {
        "message_start" => {
            if let Some(usage) = v.get("message").and_then(|m| m.get("usage")) {
                let input_tokens = usage
                    .get("input_tokens")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0);
                vec![StreamEvent::Usage {
                    stats: UsageStats {
                        input_tokens,
                        output_tokens: 0,
                    },
                }]
            } else {
                vec![]
            }
        }

        "content_block_start" => {
            let Some(cb) = v.get("content_block") else {
                return vec![];
            };
            match cb.get("type").and_then(|t| t.as_str()) {
                Some("tool_use") => {
                    let id = cb.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                    let name = cb.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                    vec![StreamEvent::ToolUseStart {
                        id: id.to_string(),
                        name: name.to_string(),
                    }]
                }
                Some("text") => {
                    let text = cb.get("text").and_then(|v| v.as_str()).unwrap_or_default();
                    if text.is_empty() {
                        vec![]
                    } else {
                        vec![StreamEvent::ContentDelta {
                            text: text.to_string(),
                        }]
                    }
                }
                Some("thinking") => {
                    // Thinking block start — no initial content to emit
                    vec![]
                }
                _ => vec![],
            }
        }

        "content_block_delta" => {
            let Some(delta) = v.get("delta") else {
                return vec![];
            };
            match delta.get("type").and_then(|t| t.as_str()) {
                Some("text_delta") => {
                    if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                        vec![StreamEvent::ContentDelta {
                            text: text.to_string(),
                        }]
                    } else {
                        vec![]
                    }
                }
                Some("input_json_delta") => {
                    if let Some(json) = delta.get("partial_json").and_then(|j| j.as_str()) {
                        vec![StreamEvent::ToolUseDelta {
                            json_chunk: json.to_string(),
                        }]
                    } else {
                        vec![]
                    }
                }
                Some("thinking_delta") => {
                    if let Some(text) = delta.get("thinking").and_then(|t| t.as_str()) {
                        vec![StreamEvent::ThinkingDelta {
                            text: text.to_string(),
                        }]
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            }
        }

        "content_block_stop" => vec![],

        "message_delta" => {
            let mut events = Vec::new();

            // Extract output tokens from usage
            let output_tokens = v
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            if output_tokens > 0 {
                events.push(StreamEvent::Usage {
                    stats: UsageStats {
                        input_tokens: 0,
                        output_tokens,
                    },
                });
            }

            // Extract stop reason
            if let Some(reason_str) = v
                .get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|r| r.as_str())
            {
                let reason = match reason_str {
                    "end_turn" => StopReason::EndTurn,
                    "tool_use" => StopReason::ToolUse,
                    "max_tokens" => StopReason::MaxTokens,
                    _ => StopReason::EndTurn,
                };
                events.push(StreamEvent::Stop { reason });
            }

            events
        }

        "error" => {
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown API error");
            vec![StreamEvent::Error {
                message: msg.to_string(),
            }]
        }

        _ => vec![],
    }
}

pub fn parse_sse_lines(text: &str) -> Vec<(String, String)> {
    let mut events = Vec::new();
    let mut current_event = String::new();
    let mut current_data = String::new();

    for line in text.lines() {
        if line.is_empty() {
            if !current_data.is_empty() {
                events.push((current_event.clone(), current_data.clone()));
                current_event.clear();
                current_data.clear();
            }
            continue;
        }
        if let Some(val) = line.strip_prefix("event: ") {
            current_event = val.to_string();
        } else if let Some(val) = line.strip_prefix("data: ") {
            current_data = val.to_string();
        }
    }

    if !current_data.is_empty() {
        events.push((current_event, current_data));
    }

    events
}
