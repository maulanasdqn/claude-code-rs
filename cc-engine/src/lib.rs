use std::sync::Arc;

use cc_errors::{AppError, AppResult};
use cc_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionDecision, Provider,
    StopReason, StreamEvent,
};
use cc_tools::ToolRegistry;
use futures::StreamExt;

/// Events emitted by the engine to callers for live display.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    TextDelta(String),
    ToolStart { name: String, id: String },
    ToolInput { json_chunk: String },
    ToolResult { name: String, output: String, is_error: bool },
    TurnComplete,
    Error(String),
}

pub struct QueryEngine {
    provider: Arc<dyn Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    max_turns: usize,
}

impl QueryEngine {
    pub fn new(
        provider: Arc<dyn Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
    ) -> Self {
        Self {
            provider,
            registry,
            permission,
            max_turns: 20,
        }
    }

    /// Run the agentic loop. Calls `on_event` for each streaming event so callers
    /// can display progress in real time.
    pub async fn run<F>(
        &self,
        mut conversation: Conversation,
        mut on_event: F,
    ) -> AppResult<Conversation>
    where
        F: FnMut(EngineEvent) + Send,
    {
        let tools = self.registry.tool_definitions();

        for turn in 0..self.max_turns {
            tracing::info!(turn, "starting provider turn");

            let mut stream = self.provider.stream(&conversation, &tools).await?;

            let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
            let mut current_tool_id = String::new();
            let mut current_tool_name = String::new();
            let mut current_tool_json = String::new();
            let mut text_buf = String::new();
            let mut stop_reason = StopReason::EndTurn;

            while let Some(event) = stream.next().await {
                match event {
                    StreamEvent::ContentDelta { text } => {
                        on_event(EngineEvent::TextDelta(text.clone()));
                        text_buf.push_str(&text);
                    }
                    StreamEvent::ToolUseStart { id, name } => {
                        if !text_buf.is_empty() {
                            assistant_blocks.push(ContentBlock::Text {
                                text: std::mem::take(&mut text_buf),
                            });
                        }
                        on_event(EngineEvent::ToolStart {
                            name: name.clone(),
                            id: id.clone(),
                        });
                        current_tool_id = id;
                        current_tool_name = name;
                        current_tool_json.clear();
                    }
                    StreamEvent::ToolUseDelta { json_chunk } => {
                        on_event(EngineEvent::ToolInput {
                            json_chunk: json_chunk.clone(),
                        });
                        current_tool_json.push_str(&json_chunk);
                    }
                    StreamEvent::Stop { reason } => {
                        stop_reason = reason;
                    }
                    StreamEvent::Error { message } => {
                        on_event(EngineEvent::Error(message.clone()));
                        return Err(AppError::Provider(message));
                    }
                }
            }

            // Flush remaining text
            if !text_buf.is_empty() {
                assistant_blocks.push(ContentBlock::Text { text: text_buf });
            }

            // Flush any pending tool use
            if !current_tool_id.is_empty() {
                let input: serde_json::Value = serde_json::from_str(&current_tool_json)
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                assistant_blocks.push(ContentBlock::ToolUse {
                    id: current_tool_id,
                    name: current_tool_name,
                    input,
                });
            }

            conversation.push(Message::assistant(assistant_blocks.clone()));

            if !matches!(stop_reason, StopReason::ToolUse) {
                on_event(EngineEvent::TurnComplete);
                return Ok(conversation);
            }

            // Execute tool calls
            let tool_uses: Vec<_> = assistant_blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::ToolUse { id, name, input } => Some((id, name, input)),
                    _ => None,
                })
                .collect();

            let mut tool_results = Vec::new();
            for (id, name, input) in tool_uses {
                let result = self.execute_tool(name, input).await;
                match result {
                    Ok(output) => {
                        on_event(EngineEvent::ToolResult {
                            name: name.clone(),
                            output: output.clone(),
                            is_error: false,
                        });
                        tool_results.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: output,
                            is_error: None,
                        });
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        on_event(EngineEvent::ToolResult {
                            name: name.clone(),
                            output: msg.clone(),
                            is_error: true,
                        });
                        tool_results.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: msg,
                            is_error: Some(true),
                        });
                    }
                }
            }

            conversation.push(Message {
                role: cc_types::Role::User,
                content: tool_results,
            });
        }

        Err(AppError::MaxTurnsExceeded(self.max_turns))
    }

    async fn execute_tool(&self, name: &str, input: &serde_json::Value) -> AppResult<String> {
        let tool = self
            .registry
            .get(name)
            .ok_or_else(|| AppError::Tool(format!("unknown tool: {name}")))?;

        if tool.permission_level() == cc_types::PermissionLevel::Dangerous {
            let decision = self.permission.check(name, input).await?;
            if let PermissionDecision::Deny(reason) = decision {
                return Err(AppError::PermissionDenied(reason));
            }
        }

        tool.execute(input.clone()).await
    }
}
