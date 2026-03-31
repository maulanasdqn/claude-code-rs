use cc_errors::AppResult;
use futures::stream::BoxStream;
use serde_json::Value;

use super::message::Conversation;

#[derive(Debug, Clone)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    ContentDelta { text: String },
    ToolUseStart { id: String, name: String },
    ToolUseDelta { json_chunk: String },
    Stop { reason: StopReason },
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
