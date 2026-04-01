use std::sync::{Arc, atomic::AtomicU8};

use claude_rust_config::HooksConfig;
use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionLevel, PermissionMode,
    Provider, Role, StopReason,
};
use claude_rust_tools::ToolRegistry;

use crate::application::undo::UndoStack;
use crate::domain::EngineEvent;
use super::compactor::compact;
use super::hook_runner::{run_post_tool_use, run_pre_tool_use, run_stop_hooks};
use super::stream_reader::read_stream;
use super::tool_executor::{execute_tool, is_overloaded};

pub struct QueryEngine {
    provider: Arc<dyn Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    hooks: HooksConfig,
    max_turns: usize,
    context_limit: u64,
    mode: Arc<AtomicU8>,
    undo_stack: Arc<UndoStack>,
}

impl QueryEngine {
    pub fn new(
        provider: Arc<dyn Provider>,
        registry: Arc<ToolRegistry>,
        permission: Arc<dyn PermissionChecker>,
        mode: Arc<AtomicU8>,
        hooks: HooksConfig,
    ) -> Self {
        Self {
            provider, registry, permission, hooks, mode,
            max_turns: 20, context_limit: 180_000,
            undo_stack: Arc::new(UndoStack::default()),
        }
    }

    pub fn with_max_turns(mut self, n: usize) -> Self {
        self.max_turns = n;
        self
    }

    pub fn mode_flag(&self) -> Arc<AtomicU8> { self.mode.clone() }
    pub fn undo_stack(&self) -> Arc<UndoStack> { self.undo_stack.clone() }

    pub async fn run<F>(
        &self,
        mut conversation: Conversation,
        mut on_event: F,
    ) -> AppResult<Conversation>
    where
        F: FnMut(EngineEvent) + Send,
    {
        let mut last_input_tokens: u64 = 0;

        for turn in 0..self.max_turns {
            tracing::info!(turn, "starting provider turn");
            let is_plan = PermissionMode::load(&self.mode) == PermissionMode::Plan;
            let tools = if is_plan { self.registry.tool_definitions_filtered(|t| t.permission_level() == PermissionLevel::ReadOnly || t.name() == "exit_plan_mode") } else { self.registry.tool_definitions() };

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
                let stop_out = run_stop_hooks(&self.hooks).await;
                if !stop_out.is_empty() {
                    on_event(EngineEvent::HookOutput { source: "stop".into(), output: stop_out });
                }
                return Ok(conversation);
            }

            let tool_uses: Vec<(String, String, serde_json::Value)> = assistant_blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::ToolUse { id, name, input } => Some((id.clone(), name.clone(), input.clone())),
                    _ => None,
                })
                .collect();

            let mut pre_outs = Vec::new();
            for (_, name, input) in &tool_uses {
                let pre = run_pre_tool_use(&self.hooks, name, &input.to_string()).await;
                pre_outs.push(pre);
            }

            let registry = self.registry.clone();
            let permission = self.permission.clone();
            let undo = self.undo_stack.clone();
            let handles: Vec<_> = tool_uses.iter()
                .zip(pre_outs.iter())
                .map(|((_, name, input), pre)| {
                    if pre.blocked {
                        tokio::spawn(async { Ok::<String, AppError>(String::new()) })
                    } else {
                        let registry = registry.clone();
                        let permission = permission.clone();
                        let undo = undo.clone();
                        let name = name.clone();
                        let input = input.clone();
                        tokio::spawn(async move {
                            execute_tool(&registry, &permission, &name, &input, &undo).await
                        })
                    }
                })
                .collect();
            let exec_results = futures::future::join_all(handles).await;

            let mut tool_results = Vec::new();
            let mut exit_plan_called = false;
            let mut pre_iter = pre_outs.into_iter();
            let mut exec_iter = exec_results.into_iter();
            for (id, name, input) in &tool_uses {
                let pre = pre_iter.next().unwrap();
                let exec_result = exec_iter.next().unwrap();
                let input_json = input.to_string();
                if !pre.output.is_empty() {
                    on_event(EngineEvent::HookOutput { source: "pre-tool".into(), output: pre.output });
                }
                if pre.blocked {
                    on_event(EngineEvent::ToolResult { name: name.clone(), output: pre.reason.clone(), is_error: true });
                    tool_results.push(ContentBlock::ToolResult { tool_use_id: id.clone(), content: pre.reason, is_error: Some(true) });
                } else {
                    let result: AppResult<String> = match exec_result {
                        Ok(r) => r,
                        Err(e) => Err(AppError::Tool(e.to_string())),
                    };
                    match result {
                        Ok(output) => {
                            let post = run_post_tool_use(&self.hooks, name, &input_json, &output).await;
                            if !post.is_empty() {
                                on_event(EngineEvent::HookOutput { source: "post-tool".into(), output: post });
                            }
                            on_event(EngineEvent::ToolResult { name: name.clone(), output: output.clone(), is_error: false });
                            tool_results.push(ContentBlock::ToolResult { tool_use_id: id.clone(), content: output, is_error: None });
                        }
                        Err(ref e) if e.is_interrupted() => {
                            return Err(AppError::Interrupted);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            on_event(EngineEvent::ToolResult { name: name.clone(), output: msg.clone(), is_error: true });
                            tool_results.push(ContentBlock::ToolResult { tool_use_id: id.clone(), content: msg, is_error: Some(true) });
                        }
                    }
                }

                if name == "enter_plan_mode" {
                    PermissionMode::Plan.store(&self.mode);
                    on_event(EngineEvent::ModeChanged { mode: PermissionMode::Plan });
                } else if name == "exit_plan_mode" {
                    PermissionMode::Normal.store(&self.mode);
                    on_event(EngineEvent::ModeChanged { mode: PermissionMode::Normal });
                    exit_plan_called = true;
                }
            }

            conversation.push(Message {
                role: Role::User,
                content: tool_results,
            });

            if exit_plan_called {
                on_event(EngineEvent::TurnComplete);
                return Ok(conversation);
            }
        }

        Err(AppError::MaxTurnsExceeded(self.max_turns))
    }
}
