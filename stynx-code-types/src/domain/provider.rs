use stynx_code_errors::AppResult;
use futures::stream::BoxStream;
use serde_json::Value;

use super::message::Conversation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndTurn => f.write_str("end_turn"),
            Self::ToolUse => f.write_str("tool_use"),
            Self::MaxTokens => f.write_str("max_tokens"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsageStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    ContentDelta { text: String },
    ThinkingDelta { text: String },
    ToolUseStart { id: String, name: String },
    ToolUseDelta { json_chunk: String },
    Stop { reason: StopReason },
    Usage { stats: UsageStats },
    Error { message: String },
}

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn stream(
        &self,
        conversation: &Conversation,
        tools: &[Value],
    ) -> AppResult<BoxStream<'static, StreamEvent>>;
}
