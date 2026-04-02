use claude_rust_types::{ContentBlock, Message, Role};

/// The type of a message group.
#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {
    /// A user message followed by an assistant response.
    UserAssistant,
    /// A tool_use block followed by its tool_result.
    ToolExchange,
    /// A system-level or standalone message.
    System,
}

/// A group of messages that form an atomic compaction unit.
#[derive(Debug, Clone)]
pub struct MessageGroup {
    pub messages: Vec<Message>,
    pub total_tokens: u64,
    pub group_type: GroupType,
}

/// Groups consecutive messages into atomic units for compaction.
///
/// - Consecutive User -> Assistant pairs become `UserAssistant` groups.
/// - Messages containing tool_use followed by messages containing tool_result
///   become `ToolExchange` groups.
/// - Everything else becomes a `System` group.
pub fn group_messages(messages: &[Message]) -> Vec<MessageGroup> {
    let mut groups = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];

        // Check if this is a tool exchange: assistant with tool_use followed by user with tool_result
        if msg.role == Role::Assistant && has_tool_use(&msg.content) {
            if i + 1 < messages.len()
                && messages[i + 1].role == Role::User
                && has_tool_result(&messages[i + 1].content)
            {
                let token_estimate = estimate_tokens(&msg.content) + estimate_tokens(&messages[i + 1].content);
                groups.push(MessageGroup {
                    messages: vec![msg.clone(), messages[i + 1].clone()],
                    total_tokens: token_estimate,
                    group_type: GroupType::ToolExchange,
                });
                i += 2;
                continue;
            }
        }

        // Check for user -> assistant pair
        if msg.role == Role::User && !has_tool_result(&msg.content) {
            if i + 1 < messages.len() && messages[i + 1].role == Role::Assistant {
                let token_estimate = estimate_tokens(&msg.content) + estimate_tokens(&messages[i + 1].content);
                groups.push(MessageGroup {
                    messages: vec![msg.clone(), messages[i + 1].clone()],
                    total_tokens: token_estimate,
                    group_type: GroupType::UserAssistant,
                });
                i += 2;
                continue;
            }
        }

        // Standalone message
        let token_estimate = estimate_tokens(&msg.content);
        groups.push(MessageGroup {
            messages: vec![msg.clone()],
            total_tokens: token_estimate,
            group_type: GroupType::System,
        });
        i += 1;
    }

    groups
}

fn has_tool_use(blocks: &[ContentBlock]) -> bool {
    blocks.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. }))
}

fn has_tool_result(blocks: &[ContentBlock]) -> bool {
    blocks.iter().any(|b| matches!(b, ContentBlock::ToolResult { .. }))
}

/// Rough token estimate: ~4 chars per token.
fn estimate_tokens(blocks: &[ContentBlock]) -> u64 {
    let chars: usize = blocks
        .iter()
        .map(|b| match b {
            ContentBlock::Text { text } => text.len(),
            ContentBlock::ToolUse { name, input, .. } => name.len() + input.to_string().len(),
            ContentBlock::ToolResult { content, .. } => content.len(),
            ContentBlock::Thinking { thinking } => thinking.len(),
            ContentBlock::Image { data, .. } => data.len(),
        })
        .sum();
    (chars as u64) / 4
}
