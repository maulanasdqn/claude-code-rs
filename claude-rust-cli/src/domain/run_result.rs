use claude_rust_types::Conversation;

#[allow(dead_code)]
pub struct RunResult {
    pub conversation: Conversation,
    pub final_text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}
