use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionLevel, PermissionMode,
    Provider, Role, StopReason,
};
use claude_rust_tools::ToolRegistry;

use crate::domain::EngineEvent;
use super::compactor::compact;
use super::stream_reader::read_stream;
use super::tool_executor::{execute_tool, is_overloaded};

pub struct QueryEngine {
    provider: Arc<dyn Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    max_turns: usize,
    context_limit: u64,
    mode: Arc<AtomicU8>,
}

impl QueryEngine {
    pub fn new(
        provider: Arc<dyn Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
    ) -> Self {
        Self {
            provider,
            registry,
            permission,
            max_turns: 20,
            context_limit: 180_000,
            mode,
        }
    }

    pub fn mode_flag(&self) -> Arc<AtomicU8> {
        self.mode.clone()
    }

    pub async fn run<F>(
        &self,
        mut conversation: Conversation,
        mut on_event: F,
    ) -> AppResult<Conversation>
    where
        F: FnMut(EngineEvent) + Send,
    {
        let tools = if PermissionMode::load(&self.mode) == PermissionMode::Plan {
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

            if last_input_tokens > 0
                && last_input_tokens > self.context_limit * 80 / 100
                && conversation.messages.len() > 2
            {
                let original_turns = conversation.messages.len();
                conversation = compact(&self.provider, conversation, &mut on_event).await?;
                on_event(EngineEvent::Compacted { original_turns });
            }

            let mut attempts = 0u32;
            let (assistant_blocks, stop_reason) = loop {
                let mut stream = match self.provider.stream(&conversation, &tools).await {
                    Ok(s) => s,
                    Err(e) if attempts < 3 && is_overloaded(&e.to_string()) => {
                        attempts += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    Err(e) => return Err(e),
                };

                let (blocks, stop_reason, input_tokens, stream_error) =
                    read_stream(&mut stream, &mut on_event).await;

                if input_tokens > 0 {
                    last_input_tokens = input_tokens;
                }

                if let Some(err_msg) = stream_error {
                    if attempts < 3 && is_overloaded(&err_msg) {
                        attempts += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(AppError::Provider(err_msg));
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
                let result = execute_tool(&self.registry, &self.permission, name, input).await;
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

                if name == "enter_plan_mode" {
                    PermissionMode::Plan.store(&self.mode);
                    on_event(EngineEvent::ModeChanged {
                        mode: PermissionMode::Plan,
                    });
                } else if name == "exit_plan_mode" {
                    PermissionMode::Normal.store(&self.mode);
                    on_event(EngineEvent::ModeChanged {
                        mode: PermissionMode::Normal,
                    });
                }
            }

            conversation.push(Message {
                role: Role::User,
                content: tool_results,
            });
        }

        Err(AppError::MaxTurnsExceeded(self.max_turns))
    }
}
