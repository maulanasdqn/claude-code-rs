use std::sync::{Arc, atomic::AtomicU8};

use stynx_code_config::HooksConfig;
use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{
    ContentBlock, Conversation, Message, PermissionChecker, PermissionLevel, PermissionMode,
    Provider, Role, StopReason,
};
use stynx_code_tools::ToolRegistry;

use stynx_code_compact::{FullCompactor, MicroCompactor, estimate_conversation_tokens};

use crate::application::undo::UndoStack;
use crate::domain::EngineEvent;
use super::hook_runner::{run_post_tool_use, run_pre_tool_use, run_stop_hooks};
use super::stream_reader::read_stream;
use super::retry::{MAX_ATTEMPTS, is_retryable, retry_delay, short_error};
use super::tool_executor::execute_tool;

/// Wraps a spawned concurrent-tool task so that, if the turn future is dropped —
/// e.g. the user interrupts (Esc / Ctrl-C), which drops the whole `run` future —
/// the task is `abort()`ed instead of being detached and left running against
/// the shared provider. Without this, an interrupt leaves orphaned sub-agent
/// streams alive and the session appears stuck.
struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl<T> AbortOnDrop<T> {
    /// Await the task to completion. On normal completion `self` drops and the
    /// trailing `abort()` is a no-op; if this future is dropped first, the task
    /// is aborted.
    async fn join(mut self) -> Result<T, tokio::task::JoinError> {
        (&mut self.0).await
    }
}

pub struct QueryEngine {
    provider: Arc<dyn Provider>,
    registry: Arc<ToolRegistry>,
    permission: Arc<dyn PermissionChecker>,
    hooks: HooksConfig,
    max_turns: usize,
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
            max_turns: 200,
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
            let tools = if is_plan {
                self.registry.tool_definitions_filtered(|t| {
                    t.permission_level() == PermissionLevel::ReadOnly || t.name() == "exit_plan_mode"
                })
            } else {
                self.registry.tool_definitions_filtered(|t| {
                    t.name() != "enter_plan_mode" && t.name() != "exit_plan_mode"
                })
            };

            // Staged, model-aware compaction. Real usage from the last request
            // when available; a local ~4-chars/token estimate otherwise (so the
            // first call of a run with an already-huge history still compacts).
            let limit = self.provider.context_window();
            let mut estimated = if last_input_tokens > 0 {
                last_input_tokens
            } else {
                estimate_conversation_tokens(&conversation)
            };
            if estimated > limit * 70 / 100 && conversation.messages.len() > 2 {
                // Stage 1 (cheap, local): truncate old tool results, keep the
                // recent exchanges verbatim.
                conversation = MicroCompactor::default().compact_conversation(&conversation);
                estimated = estimate_conversation_tokens(&conversation);
                // Stage 2 (lossy, last resort): summarize via the provider.
                if estimated > limit * 85 / 100 {
                    let original_turns = conversation.messages.len();
                    conversation = FullCompactor::new()
                        .compact(&conversation, self.provider.as_ref())
                        .await?;
                    on_event(EngineEvent::Compacted { original_turns });
                }
                last_input_tokens = 0;
            }

            let mut attempts = 0u32;
            let (assistant_blocks, stop_reason) = loop {
                let mut stream = match self.provider.stream(&conversation, &tools).await {
                    Ok(s) => s,
                    Err(e) if attempts < MAX_ATTEMPTS && is_retryable(&e.to_string()) => {
                        attempts += 1;
                        let msg = e.to_string();
                        let delay = retry_delay(attempts, &msg);
                        tracing::warn!(?delay, attempt = attempts, "provider overloaded, retrying");
                        on_event(EngineEvent::RetryNotice {
                            attempt: attempts,
                            max_attempts: MAX_ATTEMPTS,
                            delay_ms: delay.as_millis() as u64,
                            message: short_error(&msg),
                        });
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
                    if attempts < MAX_ATTEMPTS && is_retryable(&err_msg) {
                        attempts += 1;
                        let delay = retry_delay(attempts, &err_msg);
                        tracing::warn!(?delay, attempt = attempts, "stream error, retrying");
                        on_event(EngineEvent::RetryNotice {
                            attempt: attempts,
                            max_attempts: MAX_ATTEMPTS,
                            delay_ms: delay.as_millis() as u64,
                            message: short_error(&err_msg),
                        });
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

            let mut exec_results: Vec<Result<Result<String, AppError>, tokio::task::JoinError>> =
                Vec::with_capacity(tool_uses.len());

            let mut parallel_handles: Vec<(usize, AbortOnDrop<Result<String, AppError>>)> = Vec::new();
            for (i, ((_, name, input), pre)) in tool_uses.iter().zip(pre_outs.iter()).enumerate() {
                if pre.blocked {
                    continue;
                }
                let tool = registry.get(name);
                let is_safe = tool.is_some_and(|t| t.is_concurrent_safe(input));
                if is_safe {
                    let reg = registry.clone();
                    let perm = permission.clone();
                    let ud = undo.clone();
                    let n = name.clone();
                    let inp = input.clone();
                    parallel_handles.push((i, AbortOnDrop(tokio::spawn(async move {
                        execute_tool(&reg, &perm, &n, &inp, &ud).await
                    }))));
                }
            }

            let parallel_results: Vec<_> = futures::future::join_all(
                parallel_handles.into_iter().map(|(i, h)| async move { (i, h.join().await) })
            ).await;
            let mut result_map: std::collections::HashMap<usize, Result<Result<String, AppError>, tokio::task::JoinError>> =
                parallel_results.into_iter().collect();

            for (i, ((_, name, input), pre)) in tool_uses.iter().zip(pre_outs.iter()).enumerate() {
                if pre.blocked {
                    exec_results.push(Ok(Ok(String::new())));
                } else if let Some(result) = result_map.remove(&i) {
                    exec_results.push(result);
                } else {
                    // Run the tool with a streaming sink scoped so incremental
                    // output (e.g. bash stdout) flows to the UI as it arrives.
                    let (otx, mut orx) = tokio::sync::mpsc::unbounded_channel::<String>();
                    let exec = stynx_code_types::domain::tool_stream::TOOL_STREAM
                        .scope(otx, execute_tool(&registry, &permission, name, input, &undo));
                    tokio::pin!(exec);
                    let result = loop {
                        tokio::select! {
                            biased;
                            Some(chunk) = orx.recv() => {
                                on_event(EngineEvent::ToolOutput { name: name.clone(), chunk });
                            }
                            r = &mut exec => {
                                while let Ok(chunk) = orx.try_recv() {
                                    on_event(EngineEvent::ToolOutput { name: name.clone(), chunk });
                                }
                                break r;
                            }
                        }
                    };
                    exec_results.push(Ok(result));
                }
            }

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
                            if name == "exit_plan_mode" {
                                PermissionMode::Normal.store(&self.mode);
                                on_event(EngineEvent::ModeChanged { mode: PermissionMode::Normal });
                                exit_plan_called = true;
                            }
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

#[cfg(test)]
mod tests {
    use super::AbortOnDrop;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn abort_on_drop_cancels_a_running_task() {
        let completed = Arc::new(AtomicBool::new(false));
        let flag = completed.clone();
        let guard = AbortOnDrop(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            flag.store(true, Ordering::SeqCst);
        }));

        // Simulate the turn future being dropped by an interrupt.
        drop(guard);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert!(
            !completed.load(Ordering::SeqCst),
            "dropped task must be aborted, not left running to completion",
        );
    }

    #[tokio::test]
    async fn join_yields_the_value_on_normal_completion() {
        let guard = AbortOnDrop(tokio::spawn(async { 42u8 }));
        assert_eq!(guard.join().await.unwrap(), 42);
    }
}
