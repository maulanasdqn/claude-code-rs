use std::sync::Arc;

use stynx_code_errors::AppResult;
use stynx_code_types::{ContentBlock, Conversation, Message, Provider, Role, StreamEvent};
use futures::StreamExt;

use crate::domain::EngineEvent;

pub async fn compact<F>(
    provider: &Arc<dyn Provider>,
    conversation: Conversation,
    on_event: &mut F,
) -> AppResult<Conversation>
where
    F: FnMut(EngineEvent) + Send,
{
    let mut summary_parts = Vec::new();
    for msg in &conversation.messages {
        let role = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        for block in &msg.content {
            match block {
                ContentBlock::Text { text } => {
                    summary_parts.push(format!("{role}: {text}"));
                }
                ContentBlock::ToolUse { name, .. } => {
                    summary_parts.push(format!("{role}: [used tool: {name}]"));
                }
                ContentBlock::ToolResult { content, .. } => {
                    let preview = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.clone()
                    };
                    summary_parts.push(format!("{role}: [tool result: {preview}]"));
                }
                ContentBlock::Thinking { thinking } => {
                    let preview = if thinking.len() > 200 {
                        format!("{}...", &thinking[..200])
                    } else {
                        thinking.clone()
                    };
                    summary_parts.push(format!("{role}: [thinking: {preview}]"));
                }
                ContentBlock::Image { .. } => {
                    summary_parts.push(format!("{role}: [image]"));
                }
            }
        }
    }

    let summary_request = format!(
        "Please provide a concise summary of the following conversation so far. \
         Focus on key decisions, facts, and context that would be needed to continue the conversation:\n\n{}",
        summary_parts.join("\n")
    );

    let summary_request = if summary_request.len() > 50_000 {
        format!("{}...\n(truncated)", &summary_request[..50_000])
    } else {
        summary_request
    };

    let mut summary_conv = Conversation {
        system: Some("You are a summarization assistant. Provide a concise summary of the conversation.".into()),
        ..Default::default()
    };
    summary_conv.push(Message::user(&summary_request));

    let tools: Vec<serde_json::Value> = vec![];
    let mut summary_text = String::new();

    // Guard every read with an idle timeout, mirroring `read_stream`. Compaction
    // fires precisely when the context is large, so a stalled summary stream here
    // would otherwise hang the whole engine (the classic "context is big → stuck"
    // freeze). On a stall, fall back to a cheap local compaction instead.
    let idle_timeout = std::time::Duration::from_secs(180);
    match provider.stream(&summary_conv, &tools).await {
        Ok(mut stream) => loop {
            match tokio::time::timeout(idle_timeout, stream.next()).await {
                Ok(Some(StreamEvent::ContentDelta { text })) => summary_text.push_str(&text),
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_) => {
                    on_event(EngineEvent::Error(
                        "compact stream idle for 180s — using local fallback".to_string(),
                    ));
                    return Ok(fallback_compact(conversation));
                }
            }
        },
        Err(e) => {
            on_event(EngineEvent::Error(format!("compact failed: {e}")));
            return Ok(fallback_compact(conversation));
        }
    }

    if summary_text.is_empty() {
        summary_text = "Previous conversation context was compacted.".into();
    }

    let mut compacted = Conversation {
        system: conversation.system,
        ..Default::default()
    };
    compacted.push(Message::user(format!(
        "[Context from previous conversation]\n{summary_text}"
    )));

    Ok(compacted)
}

fn fallback_compact(conversation: Conversation) -> Conversation {
    let mut compacted = Conversation {
        system: conversation.system,
        ..Default::default()
    };
    let msgs = &conversation.messages;
    let start = msgs.len().saturating_sub(6);
    let first_user = msgs[start..]
        .iter()
        .position(|m| matches!(m.role, Role::User))
        .map(|i| start + i)
        .unwrap_or(start);
    for msg in &msgs[first_user..] {
        compacted.push(msg.clone());
    }
    compacted
}
