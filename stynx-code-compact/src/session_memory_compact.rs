use stynx_code_types::{ContentBlock, Conversation};
use regex::Regex;

use crate::text::safe_truncate;

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

                        if *is_error == Some(true) {
                            let preview = if content.len() > 200 {
                                format!("{}...", safe_truncate(content, 200))
                            } else {
                                content.clone()
                            };
                            memories.push(format!("Error encountered: {preview}"));
                        }

                        self.extract_file_paths(content, &mut memories);
                    }
                    _ => {}
                }
            }
        }

        memories.dedup();

        (memories, conversation.clone())
    }

    fn extract_memories_from_text(&self, text: &str, memories: &mut Vec<String>) {

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
                        format!("{}...", safe_truncate(trimmed, 200))
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

                if p.len() > 3 && !p.starts_with("//") {
                    memories.push(format!("File: {p}"));
                }
            }
        }
    }
}
