use claude_rust_types::Conversation;
use crate::domain::CommandResult;

pub fn handle_clear() -> CommandResult {
    CommandResult::ReplaceConversation(Conversation::default())
}
