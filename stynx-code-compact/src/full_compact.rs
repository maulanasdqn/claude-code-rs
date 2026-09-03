use stynx_code_errors::AppResult;
use stynx_code_types::{ContentBlock, Conversation, Message, Provider, Role, StreamEvent};
use futures::StreamExt;

use crate::prompt;
use crate::text::safe_truncate;

/// How many trailing messages survive a full compaction verbatim.
const KEEP_SUFFIX: usize = 4;

pub struct FullCompactor;

impl Default for FullCompactor {
    fn default() -> Self {
        Self
    }
}

impl FullCompactor {
    pub fn new() -> Self {
        Self
    }

    pub async fn compact(
        &self,
        conversation: &Conversation,
        provider: &dyn Provider,
    ) -> AppResult<Conversation> {
        let conversation_text = self.build_conversation_text(conversation);

        let conversation_text = if conversation_text.len() > 100_000 {
            format!("{}...\n(truncated)", safe_truncate(&conversation_text, 100_000))
        } else {
            conversation_text
        };

        let mut summary_conv = Conversation {
            system: Some(prompt::compaction_system_prompt()),
            ..Default::default()
        };
        summary_conv.push(Message::user(prompt::compaction_user_prompt(
            &conversation_text,
        )));

        let tools: Vec<serde_json::Value> = vec![];
        let mut summary_text = String::new();

        // Guard every read with an idle timeout: full compaction fires precisely
        // when the context is large, so a stalled summary stream here would
        // otherwise hang the whole engine. On a stall, fall back to a cheap
        // local compaction instead.
        let idle_timeout = std::time::Duration::from_secs(180);
        match provider.stream(&summary_conv, &tools).await {
            Ok(mut stream) => loop {
                match tokio::time::timeout(idle_timeout, stream.next()).await {
                    Ok(Some(StreamEvent::ContentDelta { text })) => summary_text.push_str(&text),
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(_) => {
                        tracing::warn!("compaction summary stream stalled — using local fallback");
                        return Ok(self.fallback_compact(conversation));
                    }
                }
            },
            Err(e) => {
                tracing::error!("full compaction failed: {e}");
                return Ok(self.fallback_compact(conversation));
            }
        }

        if summary_text.is_empty() {
            summary_text = "Previous conversation context was compacted.".into();
        }

        let mut compacted = Conversation {
            system: conversation.system.clone(),
            ..Default::default()
        };
        compacted.push(Message::user(format!(
            "[Context from previous conversation]\n{summary_text}"
        )));

        // Keep the freshest exchanges verbatim so the model doesn't lose the
        // task currently in flight to a lossy summary.
        let suffix = clean_suffix(&conversation.messages, KEEP_SUFFIX);
        if let Some(first) = suffix.first()
            && first.role == Role::User
        {
            // Preserve user/assistant alternation between summary and suffix.
            compacted.push(Message::assistant(vec![ContentBlock::Text {
                text: "I understand. I have the context from our previous conversation and will continue from there.".into(),
            }]));
        }
        for msg in suffix {
            compacted.push(msg.clone());
        }

        Ok(compacted)
    }

    fn build_conversation_text(&self, conversation: &Conversation) -> String {
        let mut parts = Vec::new();

        for msg in &conversation.messages {
            let role = match msg.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
            };
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        parts.push(format!("{role}: {text}"));
                    }
                    ContentBlock::ToolUse { name, .. } => {
                        parts.push(format!("{role}: [used tool: {name}]"));
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        parts.push(format!("{role}: [tool result: {}]", preview(content)));
                    }
                    ContentBlock::Thinking { thinking } => {
                        parts.push(format!("{role}: [thinking: {}]", preview(thinking)));
                    }
                    ContentBlock::Image { .. } => {
                        parts.push(format!("{role}: [image]"));
                    }
                }
            }
        }

        parts.join("\n")
    }

    fn fallback_compact(&self, conversation: &Conversation) -> Conversation {
        let mut compacted = Conversation {
            system: conversation.system.clone(),
            ..Default::default()
        };
        for msg in clean_suffix(&conversation.messages, 6) {
            compacted.push(msg.clone());
        }
        compacted
    }
}

fn preview(s: &str) -> String {
    if s.len() > 400 {
        format!("{}...", safe_truncate(s, 400))
    } else {
        s.to_string()
    }
}

/// The last `max` messages, advanced past any leading user message carrying
/// tool_results: a tool_result must be preceded by the assistant message with
/// the matching tool_use, and a suffix that cuts that pair in half makes the
/// provider reject the whole request.
fn clean_suffix(messages: &[Message], max: usize) -> &[Message] {
    let mut start = messages.len().saturating_sub(max);
    while start < messages.len() && is_tool_result_user(&messages[start]) {
        start += 1;
    }
    &messages[start..]
}

fn is_tool_result_user(msg: &Message) -> bool {
    msg.role == Role::User
        && msg
            .content
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolResult { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_text() -> Message {
        Message::user("hello")
    }

    fn assistant_tool_use() -> Message {
        Message::assistant(vec![ContentBlock::ToolUse {
            id: "t1".into(),
            name: "bash".into(),
            input: serde_json::json!({}),
        }])
    }

    fn user_tool_result() -> Message {
        Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "t1".into(),
                content: "ok".into(),
                is_error: None,
            }],
        }
    }

    #[test]
    fn suffix_never_starts_with_orphan_tool_result() {
        let msgs = vec![
            user_text(),
            assistant_tool_use(),
            user_tool_result(),
            assistant_tool_use(),
            user_tool_result(),
        ];
        // A window of 3 starts at the orphaned tool_result (index 2);
        // it must advance to the assistant tool_use at index 3.
        let suffix = clean_suffix(&msgs, 3);
        assert_eq!(suffix.len(), 2);
        assert!(matches!(suffix[0].content[0], ContentBlock::ToolUse { .. }));
    }

    #[test]
    fn suffix_keeps_clean_window() {
        let msgs = vec![user_text(), assistant_tool_use(), user_tool_result(), user_text()];
        let suffix = clean_suffix(&msgs, 3);
        assert_eq!(suffix.len(), 3);
    }
}
