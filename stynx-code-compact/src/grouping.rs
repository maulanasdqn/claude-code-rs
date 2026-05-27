use stynx_code_types::{ContentBlock, Message, Role};

#[derive(Debug, Clone, PartialEq)]
pub enum GroupType {

    UserAssistant,

    ToolExchange,

    System,
}

#[derive(Debug, Clone)]
pub struct MessageGroup {
    pub messages: Vec<Message>,
    pub total_tokens: u64,
    pub group_type: GroupType,
}

pub fn group_messages(messages: &[Message]) -> Vec<MessageGroup> {
    let mut groups = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];

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
