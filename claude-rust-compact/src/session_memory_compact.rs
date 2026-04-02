use claude_rust_types::{ContentBlock, Conversation};
use regex::Regex;

/// Extracts session memories (key decisions, file paths, errors) before discarding
/// conversation content during compaction.
pub struct SessionMemoryCompactor;

impl Default for SessionMemoryCompactor {
    fn default() -> Self {
        Self
    }
}

impl SessionMemoryCompactor {
    pub fn new() -> Self {
        Self
    }

    /// Extract memory strings from the conversation and return a compacted conversation.
    ///
    /// Returns `(memories, compacted_conversation)` where memories contains key decisions,
    /// file paths mentioned, and errors encountered.
    pub fn extract_and_compact(
        &self,
        conversation: &Conversation,
    ) -> (Vec<String>, Conversation) {
        let mut memories = Vec::new();

        for msg in &conversation.messages {
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        self.extract_memories_from_text(text, &mut memories);
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        // Capture error messages as memories
                        if *is_error == Some(true) {
                            let preview = if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.clone()
                            };
                            memories.push(format!("Error encountered: {preview}"));
                        }
                        // Extract file paths from tool results
                        self.extract_file_paths(content, &mut memories);
                    }
                    _ => {}
                }
            }
        }

        memories.dedup();

        // The compacted conversation is returned as-is; the caller (full_compact)
        // handles the actual message reduction. This stage only extracts memories.
        (memories, conversation.clone())
    }

    fn extract_memories_from_text(&self, text: &str, memories: &mut Vec<String>) {
        // Extract decision patterns
        let decision_patterns = [
            "I decided to",
            "The solution is",
            "We agreed to",
            "The approach is",
            "The fix is",
            "The issue was",
            "The problem was",
            "The root cause",
        ];

        for line in text.lines() {
            let trimmed = line.trim();
            for pattern in &decision_patterns {
                if trimmed.contains(pattern) {
                    let memory = if trimmed.len() > 200 {
                        format!("{}...", &trimmed[..200])
                    } else {
                        trimmed.to_string()
                    };
                    memories.push(memory);
                    break;
                }
            }
        }

        self.extract_file_paths(text, memories);
    }

    fn extract_file_paths(&self, text: &str, memories: &mut Vec<String>) {
        let path_re = Regex::new(r#"(?:^|[\s"'`(])(/[\w./-]+\.\w+)"#).unwrap();
        for cap in path_re.captures_iter(text) {
            if let Some(path) = cap.get(1) {
                let p = path.as_str();
                // Filter out obviously non-file patterns
                if p.len() > 3 && !p.starts_with("//") {
                    memories.push(format!("File: {p}"));
                }
            }
        }
    }
}
