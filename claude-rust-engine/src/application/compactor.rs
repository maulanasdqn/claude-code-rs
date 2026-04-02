use std::sync::Arc;

use claude_rust_errors::AppResult;
use claude_rust_types::{ContentBlock, Conversation, Message, Provider, Role, StreamEvent};
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

    match provider.stream(&summary_conv, &tools).await {
        Ok(mut stream) => {
            while let Some(event) = stream.next().await {
                if let StreamEvent::ContentDelta { text } = event {
                    summary_text.push_str(&text);
                }
            }
        }
        Err(e) => {
            on_event(EngineEvent::Error(format!("compact failed: {e}")));
            let mut compacted = Conversation {
                system: conversation.system,
                ..Default::default()
            };
            let keep = conversation.messages.len().min(4);
            let start = conversation.messages.len() - keep;
            for msg in &conversation.messages[start..] {
                compacted.push(msg.clone());
            }
            return Ok(compacted);
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
    compacted.push(Message::assistant(vec![ContentBlock::Text {
        text: "I understand. I have the context from our previous conversation. How can I help you next?".into(),
    }]));

    Ok(compacted)
}
