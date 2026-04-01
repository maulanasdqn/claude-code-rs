use claude_rust_types::{ContentBlock, StopReason, StreamEvent};
use futures::StreamExt;

use crate::domain::EngineEvent;

struct PendingTool {
    id: String,
    name: String,
    json: String,
}

pub async fn read_stream(
    stream: &mut (impl StreamExt<Item = StreamEvent> + Unpin),
    on_event: &mut (impl FnMut(EngineEvent) + Send),
) -> (Vec<ContentBlock>, StopReason, u64, Option<String>) {
    let mut blocks: Vec<ContentBlock> = Vec::new();
    let mut pending_tool: Option<PendingTool> = None;
    let mut text_buf = String::new();
    let mut thinking_buf = String::new();
    let mut stop_reason = StopReason::EndTurn;
    let mut stream_error: Option<String> = None;
    let mut last_input_tokens: u64 = 0;

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::ContentDelta { text } => {
                if !thinking_buf.is_empty() {
                    blocks.push(ContentBlock::Thinking {
                        thinking: std::mem::take(&mut thinking_buf),
                    });
                }
                on_event(EngineEvent::TextDelta(text.clone()));
                text_buf.push_str(&text);
            }
            StreamEvent::ThinkingDelta { text } => {
                on_event(EngineEvent::ThinkingDelta(text.clone()));
                thinking_buf.push_str(&text);
            }
            StreamEvent::ToolUseStart { id, name } => {
                if !text_buf.is_empty() {
                    blocks.push(ContentBlock::Text {
                        text: std::mem::take(&mut text_buf),
                    });
                }
                if let Some(pt) = pending_tool.take() {
                    let input: serde_json::Value = serde_json::from_str(&pt.json)
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    blocks.push(ContentBlock::ToolUse {
                        id: pt.id,
                        name: pt.name,
                        input,
                    });
                }
                on_event(EngineEvent::ToolStart {
                    name: name.clone(),
                    id: id.clone(),
                });
                pending_tool = Some(PendingTool {
                    id,
                    name,
                    json: String::new(),
                });
            }
            StreamEvent::ToolUseDelta { json_chunk } => {
                on_event(EngineEvent::ToolInput {
                    json_chunk: json_chunk.clone(),
                });
                if let Some(ref mut pt) = pending_tool {
                    pt.json.push_str(&json_chunk);
                }
            }
            StreamEvent::Stop { reason } => {
                stop_reason = reason;
            }
            StreamEvent::Usage { stats } => {
                if stats.input_tokens > 0 {
                    last_input_tokens = stats.input_tokens;
                }
                on_event(EngineEvent::Usage {
                    input_tokens: stats.input_tokens,
                    output_tokens: stats.output_tokens,
                });
            }
            StreamEvent::Error { message } => {
                stream_error = Some(message);
                break;
            }
        }
    }

    if !thinking_buf.is_empty() {
        blocks.push(ContentBlock::Thinking {
            thinking: thinking_buf,
        });
    }

    if !text_buf.is_empty() {
        blocks.push(ContentBlock::Text { text: text_buf });
    }

    if let Some(pt) = pending_tool.take() {
        let input: serde_json::Value = serde_json::from_str(&pt.json)
            .unwrap_or(serde_json::Value::Object(Default::default()));
        blocks.push(ContentBlock::ToolUse {
            id: pt.id,
            name: pt.name,
            input,
        });
    }

    (blocks, stop_reason, last_input_tokens, stream_error)
}
