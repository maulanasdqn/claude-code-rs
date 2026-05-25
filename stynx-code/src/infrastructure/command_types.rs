use stynx_code_types::Conversation;

pub enum CommandAction {
    Output(String),
    ReplaceConversation(Conversation),
    SendToEngine(String, Vec<String>),
    Quit,
    Continue,
}
