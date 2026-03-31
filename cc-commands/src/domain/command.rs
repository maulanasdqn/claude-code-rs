use cc_types::Conversation;

pub enum SlashCommand {
    Help,
    Clear,
    Compact,
    Model(String),
    Quit,
}

pub enum CommandResult {
    Output(String),
    ReplaceConversation(Conversation),
    Quit,
}
