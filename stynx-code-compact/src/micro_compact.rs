use stynx_code_types::{ContentBlock, Conversation, Message};

/// Reduces individual tool results that exceed a size threshold.
pub struct MicroCompactor {
    /// Maximum allowed size for tool result content in characters.
    pub max_tool_result_size: usize,
}

impl Default for MicroCompactor {
    fn default() -> Self {
        Self {
            max_tool_result_size: 10_000,
        }
    }
}

impl MicroCompactor {
    pub fn new(max_tool_result_size: usize) -> Self {
        Self {
            max_tool_result_size,
        }
    }

    /// Compact a single message by truncating oversized tool results.
    pub fn compact_message(&self, message: &Message) -> Message {
        let content = message
            .content
            .iter()
            .map(|block| match block {
                ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } => {
                    if content.len() > self.max_tool_result_size {
                        let original_len = content.len();
                        let truncated =
                            format!("{}...\n[truncated - {original_len} chars]", &content[..self.max_tool_result_size]);
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

    /// Compact all messages in a conversation.
    pub fn compact_conversation(&self, conversation: &Conversation) -> Conversation {
        let messages = conversation
            .messages
            .iter()
            .map(|msg| self.compact_message(msg))
            .collect();

        Conversation {
            system: conversation.system.clone(),
            messages,
        }
    }
}
