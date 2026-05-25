/// Returns the system prompt for asking the model to summarize a conversation.
pub fn compaction_system_prompt() -> String {
    "You are a summarization assistant. Your job is to produce a concise summary of a \
     conversation between a user and an AI assistant. Focus on:\n\
     - Key decisions made\n\
     - Important facts and context\n\
     - File paths and code changes discussed\n\
     - Errors encountered and their resolutions\n\
     - Any pending tasks or next steps\n\n\
     Be concise but preserve all information needed to continue the conversation."
        .to_string()
}

/// Wraps conversation text in a summarization request for the model.
pub fn compaction_user_prompt(conversation_text: &str) -> String {
    format!(
        "Please provide a concise summary of the following conversation so far. \
         Focus on key decisions, facts, and context that would be needed to continue \
         the conversation:\n\n{conversation_text}"
    )
}
