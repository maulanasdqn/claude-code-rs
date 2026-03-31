use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionDecision, PermissionLevel,
    Provider, StopReason, StreamEvent,
};
use claude_rust_tools::ToolRegistry;
use futures::StreamExt;

use crate::domain::EngineEvent;

/// A pending tool use being accumulated from the stream.
struct PendingTool {
    id: String,
    name: String,
    json: String,
}

pub struct QueryEngine {
    provider: Arc<dyn Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    max_turns: usize,
    context_limit: u64,
    plan_mode: Arc<AtomicBool>,
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
            context_limit: 180_000,
            plan_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn plan_mode_flag(&self) -> Arc<AtomicBool> {
        self.plan_mode.clone()
    }

    pub async fn run<F>(
        &self,
        mut conversation: Conversation,
        mut on_event: F,
    ) -> AppResult<Conversation>
    where
        F: FnMut(EngineEvent) + Send,
    {
        let tools = if self.plan_mode.load(Ordering::Relaxed) {
            // In plan mode, only expose ReadOnly tools + exit_plan_mode
            self.registry.tool_definitions_filtered(|tool| {
                tool.permission_level() == PermissionLevel::ReadOnly
                    || tool.name() == "exit_plan_mode"
            })
        } else {
            self.registry.tool_definitions()
        };
        let mut last_input_tokens: u64 = 0;

        for turn in 0..self.max_turns {
            tracing::info!(turn, "starting provider turn");

            // Auto-compact check: if input tokens exceed 80% of context limit
            if last_input_tokens > 0
                && last_input_tokens > self.context_limit * 80 / 100
                && conversation.messages.len() > 2
            {
                let original_turns = conversation.messages.len();
                conversation = self.compact(conversation, &mut on_event).await?;
                on_event(EngineEvent::Compacted { original_turns });
            }

            // Retry loop: covers both stream creation errors and in-stream overload errors
            let mut attempts = 0u32;
            let (assistant_blocks, stop_reason) = loop {
                let mut stream = match self.provider.stream(&conversation, &tools).await {
                    Ok(s) => s,
                    Err(e) if attempts < 3 && is_overloaded(&e.to_string()) => {
                        attempts += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        on_event(EngineEvent::Error(format!(
                            "API overloaded, retrying in {}s...",
                            delay.as_secs()
                        )));
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    Err(e) => return Err(e),
                };

                let mut blocks: Vec<ContentBlock> = Vec::new();
                let mut pending_tool: Option<PendingTool> = None;
                let mut text_buf = String::new();
                let mut thinking_buf = String::new();
                let mut stop_reason = StopReason::EndTurn;
                let mut stream_error: Option<String> = None;

                while let Some(event) = stream.next().await {
                    match event {
                        StreamEvent::ContentDelta { text } => {
                            if !thinking_buf.is_empty() {
                                blocks.push(ContentBlock::Thinking {
                                    thinking: std::mem::take(&mut thinking_buf),
                                });
                            }
                            on_event(EngineEvent::TextDelta(text.clone()));
                            text_buf.push_str(&text);
                        }
                        StreamEvent::ThinkingDelta { text } => {
                            on_event(EngineEvent::ThinkingDelta(text.clone()));
                            thinking_buf.push_str(&text);
                        }
                        StreamEvent::ToolUseStart { id, name } => {
                            if !text_buf.is_empty() {
                                blocks.push(ContentBlock::Text {
                                    text: std::mem::take(&mut text_buf),
                                });
                            }
                            if let Some(pt) = pending_tool.take() {
                                let input: serde_json::Value = serde_json::from_str(&pt.json)
                                    .unwrap_or(serde_json::Value::Object(Default::default()));
                                blocks.push(ContentBlock::ToolUse {
                                    id: pt.id,
                                    name: pt.name,
                                    input,
                                });
                            }
                            on_event(EngineEvent::ToolStart {
                                name: name.clone(),
                                id: id.clone(),
                            });
                            pending_tool = Some(PendingTool {
                                id,
                                name,
                                json: String::new(),
                            });
                        }
                        StreamEvent::ToolUseDelta { json_chunk } => {
                            on_event(EngineEvent::ToolInput {
                                json_chunk: json_chunk.clone(),
                            });
                            if let Some(ref mut pt) = pending_tool {
                                pt.json.push_str(&json_chunk);
                            }
                        }
                        StreamEvent::Stop { reason } => {
                            stop_reason = reason;
                        }
                        StreamEvent::Usage { stats } => {
                            if stats.input_tokens > 0 {
                                last_input_tokens = stats.input_tokens;
                            }
                            on_event(EngineEvent::Usage {
                                input_tokens: stats.input_tokens,
                                output_tokens: stats.output_tokens,
                            });
                        }
                        StreamEvent::Error { message } => {
                            stream_error = Some(message);
                            break;
                        }
                    }
                }

                // Handle stream errors with retry for overloaded
                if let Some(err_msg) = stream_error {
                    if attempts < 3 && is_overloaded(&err_msg) {
                        attempts += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        on_event(EngineEvent::Error(format!(
                            "API overloaded, retrying in {}s...",
                            delay.as_secs()
                        )));
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    on_event(EngineEvent::Error(err_msg.clone()));
                    return Err(AppError::Provider(err_msg));
                }

                // Flush remaining thinking
                if !thinking_buf.is_empty() {
                    blocks.push(ContentBlock::Thinking {
                        thinking: thinking_buf,
                    });
                }

                // Flush remaining text
                if !text_buf.is_empty() {
                    blocks.push(ContentBlock::Text { text: text_buf });
                }

                // Flush remaining pending tool
                if let Some(pt) = pending_tool.take() {
                    let input: serde_json::Value = serde_json::from_str(&pt.json)
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    blocks.push(ContentBlock::ToolUse {
                        id: pt.id,
                        name: pt.name,
                        input,
                    });
                }

                break (blocks, stop_reason);
            };

            conversation.push(Message::assistant(assistant_blocks.clone()));

            if !matches!(stop_reason, StopReason::ToolUse) {
                on_event(EngineEvent::TurnComplete);
                return Ok(conversation);
            }

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

                // Toggle plan mode if enter/exit plan mode tool was used
                if name == "enter_plan_mode" {
                    self.plan_mode.store(true, Ordering::Relaxed);
                    on_event(EngineEvent::PlanModeChanged { enabled: true });
                } else if name == "exit_plan_mode" {
                    self.plan_mode.store(false, Ordering::Relaxed);
                    on_event(EngineEvent::PlanModeChanged { enabled: false });
                }
            }

            conversation.push(Message {
                role: claude_rust_types::Role::User,
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

        if tool.permission_level() == claude_rust_types::PermissionLevel::Dangerous {
            let decision = self.permission.check(name, input).await?;
            if let PermissionDecision::Deny(reason) = decision {
                return Err(AppError::PermissionDenied(reason));
            }
        }

        tool.execute(input.clone()).await
    }

    async fn compact<F>(
        &self,
        conversation: Conversation,
        on_event: &mut F,
    ) -> AppResult<Conversation>
    where
        F: FnMut(EngineEvent) + Send,
    {
        // Build a summarization request
        let mut summary_parts = Vec::new();
        for msg in &conversation.messages {
            let role = match msg.role {
                claude_rust_types::Role::User => "User",
                claude_rust_types::Role::Assistant => "Assistant",
            };
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        summary_parts.push(format!("{role}: {text}"));
                    }
                    ContentBlock::ToolUse { name, .. } => {
                        summary_parts.push(format!("{role}: [used tool: {name}]"));
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        let preview = if content.len() > 200 {
                            format!("{}...", &content[..200])
                        } else {
                            content.clone()
                        };
                        summary_parts.push(format!("{role}: [tool result: {preview}]"));
                    }
                    ContentBlock::Thinking { thinking } => {
                        let preview = if thinking.len() > 200 {
                            format!("{}...", &thinking[..200])
                        } else {
                            thinking.clone()
                        };
                        summary_parts.push(format!("{role}: [thinking: {preview}]"));
                    }
                }
            }
        }

        let summary_request = format!(
            "Please provide a concise summary of the following conversation so far. \
             Focus on key decisions, facts, and context that would be needed to continue the conversation:\n\n{}",
            summary_parts.join("\n")
        );

        // Truncate if too long
        let summary_request = if summary_request.len() > 50_000 {
            format!("{}...\n(truncated)", &summary_request[..50_000])
        } else {
            summary_request
        };

        let mut summary_conv = Conversation::default();
        summary_conv.system = Some("You are a summarization assistant. Provide a concise summary of the conversation.".into());
        summary_conv.push(Message::user(&summary_request));

        let tools: Vec<serde_json::Value> = vec![];
        let mut summary_text = String::new();

        match self.provider.stream(&summary_conv, &tools).await {
            Ok(mut stream) => {
                while let Some(event) = stream.next().await {
                    if let StreamEvent::ContentDelta { text } = event {
                        summary_text.push_str(&text);
                    }
                }
            }
            Err(e) => {
                // If summarization fails, just keep recent messages
                on_event(EngineEvent::Error(format!("compact failed: {e}")));
                let mut compacted = Conversation::default();
                compacted.system = conversation.system;
                // Keep last 4 messages
                let keep = conversation.messages.len().min(4);
                let start = conversation.messages.len() - keep;
                for msg in &conversation.messages[start..] {
                    compacted.push(msg.clone());
                }
                return Ok(compacted);
            }
        }

        if summary_text.is_empty() {
            summary_text = "Previous conversation context was compacted.".into();
        }

        let mut compacted = Conversation::default();
        compacted.system = conversation.system;
        compacted.push(Message::user(&format!(
            "[Context from previous conversation]\n{summary_text}"
        )));
        compacted.push(Message::assistant(vec![ContentBlock::Text {
            text: "I understand. I have the context from our previous conversation. How can I help you next?".into(),
        }]));

        Ok(compacted)
    }
}

/// Check if an error message indicates an API overload.
fn is_overloaded(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("overloaded") || lower.contains("529") || lower.contains("rate")
}
