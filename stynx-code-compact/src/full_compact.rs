use stynx_code_errors::AppResult;
use stynx_code_types::{ContentBlock, Conversation, Message, Provider, Role, StreamEvent};
use futures::StreamExt;

use crate::prompt;

pub struct FullCompactor;

impl Default for FullCompactor {
    fn default() -> Self {
        Self
    }
}

impl FullCompactor {
    pub fn new() -> Self {
        Self
    }

    pub async fn compact(
        &self,
        conversation: &Conversation,
        provider: &dyn Provider,
    ) -> AppResult<Conversation> {

        let conversation_text = self.build_conversation_text(conversation);

        let conversation_text = if conversation_text.len() > 50_000 {
            format!("{}...\n(truncated)", &conversation_text[..50_000])
        } else {
            conversation_text
        };

        let mut summary_conv = Conversation {
            system: Some(prompt::compaction_system_prompt()),
            ..Default::default()
        };
        summary_conv.push(Message::user(prompt::compaction_user_prompt(
            &conversation_text,
        )));

        let tools: Vec<serde_json::Value> = vec![];
        let mut summary_text = String::new();

        match provider.stream(&summary_conv, &tools).await {
            Ok(mut stream) => {
                while let Some(event) = stream.next().await {
                    if let StreamEvent::ContentDelta { text } = event {
                        summary_text.push_str(&text);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Full compaction failed: {e}");

                return Ok(self.fallback_compact(conversation));
            }
        }

        if summary_text.is_empty() {
            summary_text = "Previous conversation context was compacted.".into();
        }

        let mut compacted = Conversation {
            system: conversation.system.clone(),
            ..Default::default()
        };

        compacted.push(Message::user(format!(
            "[Context from previous conversation]\n{summary_text}"
        )));
        compacted.push(Message::assistant(vec![ContentBlock::Text {
            text: "I understand. I have the context from our previous conversation. How can I help you next?".into(),
        }]));

        let keep = conversation.messages.len().min(2);
        let start = conversation.messages.len() - keep;
        for msg in &conversation.messages[start..] {
            compacted.push(msg.clone());
        }

        Ok(compacted)
    }

    fn build_conversation_text(&self, conversation: &Conversation) -> String {
        let mut parts = Vec::new();

        for msg in &conversation.messages {
            let role = match msg.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
            };
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        parts.push(format!("{role}: {text}"));
                    }
                    ContentBlock::ToolUse { name, .. } => {
                        parts.push(format!("{role}: [used tool: {name}]"));
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        let preview = if content.len() > 200 {
                            format!("{}...", &content[..200])
                        } else {
                            content.clone()
                        };
                        parts.push(format!("{role}: [tool result: {preview}]"));
                    }
                    ContentBlock::Thinking { thinking } => {
                        let preview = if thinking.len() > 200 {
                            format!("{}...", &thinking[..200])
                        } else {
                            thinking.clone()
                        };
                        parts.push(format!("{role}: [thinking: {preview}]"));
                    }
                    ContentBlock::Image { .. } => {
                        parts.push(format!("{role}: [image]"));
                    }
                }
            }
        }

        parts.join("\n")
    }

    fn fallback_compact(&self, conversation: &Conversation) -> Conversation {
        let mut compacted = Conversation {
            system: conversation.system.clone(),
            ..Default::default()
        };
        let keep = conversation.messages.len().min(4);
        let start = conversation.messages.len() - keep;
        for msg in &conversation.messages[start..] {
            compacted.push(msg.clone());
        }
        compacted
    }
}
