use claude_rust_types::Conversation;

pub enum SlashCommand {
    Help,
    Clear,
    Compact,
    Cost,
    Model(String),
    Mode,
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
