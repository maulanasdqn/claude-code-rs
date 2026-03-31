use claude_rust_types::Conversation;

pub enum SlashCommand {
    Help,
    Clear,
    Compact,
    Cost,
    Model(String),
    Diff,
    Status,
    Doctor,
    Config,
    Permissions,
    Session(String),
    Plan,
    Quit,
}

pub enum CommandResult {
    Output(String),
    ReplaceConversation(Conversation),
    Quit,
}
