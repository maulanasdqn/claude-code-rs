use stynx_code_types::Conversation;
use crate::domain::CommandResult;

pub fn handle_clear() -> CommandResult {
    CommandResult::ReplaceConversation(Conversation::default())
}
