use cc_types::{StopReason, StreamEvent};
use serde_json::Value;

pub fn parse_sse_event(event_type: &str, data: &str) -> Option<StreamEvent> {
    let v: Value = serde_json::from_str(data).ok()?;

    match event_type {
        "content_block_start" => {
            let cb = v.get("content_block")?;
            match cb.get("type")?.as_str()? {
                "tool_use" => Some(StreamEvent::ToolUseStart {
                    id: cb.get("id")?.as_str()?.to_string(),
                    name: cb.get("name")?.as_str()?.to_string(),
                }),
                "text" => {
                    let text = cb.get("text")?.as_str().unwrap_or_default();
                    if text.is_empty() {
                        None
                    } else {
                        Some(StreamEvent::ContentDelta {
                            text: text.to_string(),
                        })
                    }
                }
                "thinking" => None,
                _ => None,
            }
        }

        "content_block_delta" => {
            let delta = v.get("delta")?;
            match delta.get("type")?.as_str()? {
                "text_delta" => Some(StreamEvent::ContentDelta {
                    text: delta.get("text")?.as_str()?.to_string(),
                }),
                "input_json_delta" => Some(StreamEvent::ToolUseDelta {
                    json_chunk: delta.get("partial_json")?.as_str()?.to_string(),
                }),
                "thinking_delta" => None,
                _ => None,
            }
        }

        "content_block_stop" => None,

        "message_delta" => {
            let delta = v.get("delta")?;
            let reason = match delta.get("stop_reason")?.as_str()? {
                "end_turn" => StopReason::EndTurn,
                "tool_use" => StopReason::ToolUse,
                "max_tokens" => StopReason::MaxTokens,
                _ => StopReason::EndTurn,
            };
            Some(StreamEvent::Stop { reason })
        }

        "error" => {
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown API error");
            Some(StreamEvent::Error {
                message: msg.to_string(),
            })
        }

        _ => None,
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
