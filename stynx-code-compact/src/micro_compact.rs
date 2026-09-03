use stynx_code_types::{ContentBlock, Conversation, Message};

use crate::text::safe_truncate;

/// Cheap, local, lossless-enough compaction: truncates large tool results in
/// OLD messages while leaving the most recent exchanges untouched. Old tool
/// output is rarely needed verbatim once the conversation has moved on, and
/// this buys a lot of headroom before the expensive full summary is needed.
pub struct MicroCompactor {
    pub max_tool_result_size: usize,
    /// Messages at the tail of the conversation left untouched.
    pub keep_recent_messages: usize,
}

impl Default for MicroCompactor {
    fn default() -> Self {
        Self {
            max_tool_result_size: 2_000,
            keep_recent_messages: 6,
        }
    }
}

impl MicroCompactor {
    pub fn new(max_tool_result_size: usize, keep_recent_messages: usize) -> Self {
        Self { max_tool_result_size, keep_recent_messages }
    }

    pub fn compact_message(&self, message: &Message) -> Message {
        let content = message
            .content
            .iter()
            .map(|block| match block {
                ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                    if content.len() > self.max_tool_result_size {
                        let original_len = content.len();
                        let truncated = format!(
                            "{}...\n[truncated - {original_len} chars]",
                            safe_truncate(content, self.max_tool_result_size)
                        );
                        ContentBlock::ToolResult {
                            tool_use_id: tool_use_id.clone(),
                            content: truncated,
                            is_error: *is_error,
                        }
                    } else {
                        block.clone()
                    }
                }
                _ => block.clone(),
            })
            .collect();

        Message {
            role: message.role.clone(),
            content,
        }
    }

    pub fn compact_conversation(&self, conversation: &Conversation) -> Conversation {
        let total = conversation.messages.len();
        let cutoff = total.saturating_sub(self.keep_recent_messages);
        let messages = conversation
            .messages
            .iter()
            .enumerate()
            .map(|(i, msg)| if i < cutoff { self.compact_message(msg) } else { msg.clone() })
            .collect();

        Conversation {
            system: conversation.system.clone(),
            messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stynx_code_types::Role;

    fn tool_result_msg(size: usize) -> Message {
        Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "t1".into(),
                content: "x".repeat(size),
                is_error: None,
            }],
        }
    }

    #[test]
    fn truncates_old_but_keeps_recent() {
        let compactor = MicroCompactor::new(100, 2);
        let mut conv = Conversation::default();
        for _ in 0..4 {
            conv.push(tool_result_msg(500));
        }
        let out = compactor.compact_conversation(&conv);
        let size = |m: &Message| match &m.content[0] {
            ContentBlock::ToolResult { content, .. } => content.len(),
            _ => unreachable!(),
        };
        assert!(size(&out.messages[0]) < 200, "old result should be truncated");
        assert!(size(&out.messages[1]) < 200, "old result should be truncated");
        assert_eq!(size(&out.messages[2]), 500, "recent results untouched");
        assert_eq!(size(&out.messages[3]), 500, "recent results untouched");
    }
}
