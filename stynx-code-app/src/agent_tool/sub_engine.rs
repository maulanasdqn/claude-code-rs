use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use stynx_code_config::HooksConfig;
use stynx_code_engine::{EngineEvent, QueryEngine, sub_agent_sink};
use stynx_code_errors::AppResult;
use stynx_code_types::{Conversation, Message, PermissionChecker, Role};
use stynx_code_tools::ToolRegistry;

#[derive(Clone)]
pub(super) struct SubEngine {
    pub provider: Arc<dyn stynx_code_types::Provider>,
    pub registry: Arc<ToolRegistry>,
    pub permission: Arc<dyn PermissionChecker>,
    pub mode: Arc<AtomicU8>,
    pub hooks: HooksConfig,
}

struct Activity {
    text: String,
    actions: Vec<String>,
    current_tool: Option<(String, String)>,
    read_paths: Vec<String>,
    edited_paths: Vec<String>,
    written_paths: Vec<String>,
    bash_commands: Vec<String>,
    error_count: usize,
}

/// Called with a short description each time the sub-agent starts or finishes a
/// tool, so a background runner (e.g. the intern manager) can surface live status.
pub(super) type ActionReporter = Arc<dyn Fn(String) + Send + Sync>;

/// Wall-clock budget for a single nested sub-agent run. A nested
/// `QueryEngine::run` has no built-in bound — it can loop up to `max_turns`, and
/// a slowly-trickling provider stream never trips the per-read idle timeout — so
/// every sub-agent (explore / agent / intern) funnels through `run_bounded`.
/// This is what stops one stuck sub-agent from hanging the engine's concurrent
/// `join_all` when several run at once.
fn sub_agent_timeout() -> std::time::Duration {
    let secs = std::env::var("STYNX_SUBAGENT_TIMEOUT_SECS")
        .or_else(|_| std::env::var("INTERN_TIMEOUT_SECS"))
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(600);
    std::time::Duration::from_secs(secs)
}

impl SubEngine {
    /// Run the nested engine with the sub-agent wall-clock cap. On timeout the
    /// inner future is dropped (which cancels the nested `QueryEngine::run`, since
    /// it is awaited inline rather than spawned), the sub-agent progress row is
    /// cleared, and an `Ok` timeout marker is returned so the caller resolves and
    /// the mentor can auto-recover instead of the session freezing.
    pub(super) async fn run_bounded(
        &self,
        label: &str,
        system: &str,
        task: &str,
        reporter: Option<ActionReporter>,
    ) -> AppResult<String> {
        let dur = sub_agent_timeout();
        match tokio::time::timeout(dur, self.run(label, system, task, reporter)).await {
            Ok(r) => r,
            Err(_) => {
                tracing::warn!(sub_agent = %label, secs = dur.as_secs(), "sub-agent timed out");
                sub_agent_sink::send(EngineEvent::SubAgentDone { label: label.to_string() });
                Ok(format!(
                    "[TIMEOUT] sub-agent '{label}' did not finish in {}s — aborted. AUTO-RECOVER NOW (do not ask the user): retry with a narrower task, delegate to a different agent, or do the work inline yourself.",
                    dur.as_secs()
                ))
            }
        }
    }

    pub(super) async fn run(
        &self,
        label: &str,
        system: &str,
        task: &str,
        reporter: Option<ActionReporter>,
    ) -> AppResult<String> {
        let sub_registry = Arc::new(self.registry.clone_excluding(&["agent", "explore"]));
        let engine = QueryEngine::new(
            self.provider.clone(),
            sub_registry,
            self.permission.clone(),
            self.mode.clone(),
            self.hooks.clone(),
        );
        let mut conv = Conversation {
            system: Some(system.to_string()),
            ..Default::default()
        };
        conv.push(Message { role: Role::User, content: vec![stynx_code_types::ContentBlock::Text { text: task.to_string() }] });

        let activity = std::sync::Arc::new(std::sync::Mutex::new(Activity {
            text: String::new(),
            actions: Vec::new(),
            current_tool: None,
            read_paths: Vec::new(),
            edited_paths: Vec::new(),
            written_paths: Vec::new(),
            bash_commands: Vec::new(),
            error_count: 0,
        }));
        let act_ref = activity.clone();
        let label_for_cb = label.to_string();
        engine
            .run(conv, move |event| {
                let mut a = act_ref.lock().unwrap();
                match event {
                    EngineEvent::TextDelta(text) => a.text.push_str(&text),
                    EngineEvent::ToolStart { name, .. } => {
                        sub_agent_sink::send(EngineEvent::SubAgentProgress {
                            label: label_for_cb.clone(),
                            summary: format!("calling {name}…"),
                        });
                        if let Some(r) = &reporter {
                            r(format!("calling {name}…"));
                        }
                        a.current_tool = Some((name, String::new()));
                    }
                    EngineEvent::ToolInput { json_chunk } => {
                        if let Some((_, buf)) = a.current_tool.as_mut() {
                            buf.push_str(&json_chunk);
                        }
                    }
                    EngineEvent::ToolResult { name, output, is_error } => {
                        let input = a
                            .current_tool
                            .take()
                            .filter(|(n, _)| n == &name)
                            .map(|(_, j)| j)
                            .unwrap_or_default();
                        let summary = summarize_action(&name, &input, &output, is_error);
                        sub_agent_sink::send(EngineEvent::SubAgentProgress {
                            label: label_for_cb.clone(),
                            summary: summary.clone(),
                        });
                        if let Some(r) = &reporter {
                            r(summary.clone());
                        }
                        a.actions.push(summary);
                        if !is_error {
                            let parsed: Option<serde_json::Value> = serde_json::from_str(&input).ok();
                            let path = parsed.as_ref()
                                .and_then(|v| v.get("file_path"))
                                .and_then(|v| v.as_str())
                                .map(str::to_string);
                            match name.as_str() {
                                "read" => if let Some(p) = path { a.read_paths.push(p); },
                                "file_edit" => if let Some(p) = path { a.edited_paths.push(p); },
                                "file_write" => if let Some(p) = path { a.written_paths.push(p); },
                                "bash" => {
                                    if let Some(cmd) = parsed.as_ref().and_then(|v| v.get("command")).and_then(|v| v.as_str()) {
                                        a.bash_commands.push(cmd.to_string());
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            a.error_count += 1;
                        }
                    }
                    _ => {}
                }
            })
            .await?;

        sub_agent_sink::send(EngineEvent::SubAgentDone { label: label.to_string() });

        let act = activity.lock().unwrap();
        let has_text = !act.text.trim().is_empty();
        let took_actions = !act.actions.is_empty();

        let mut out = String::new();
        if has_text {
            out.push_str(act.text.trim());
            out.push('\n');
        }
        if took_actions {
            if !out.is_empty() { out.push('\n'); }
            out.push_str("Actions taken:\n");
            for a in &act.actions {
                out.push_str("  • ");
                out.push_str(a);
                out.push('\n');
            }
        }

        // Genuine failure is ONLY "nothing said and nothing done". An intern that
        // read files and wrote an analysis (but skipped a rigid label) did real
        // work — don't discard it. Whether the task needed file writes is the
        // mentor's call; the ground-truth block below states the facts plainly.
        if !has_text && !took_actions {
            out.push_str(
                "[FAILED] intern produced no output and took no tool actions — it stalled or refused the task. \
                 AUTO-RECOVER NOW (do not ask the user): take over yourself, or try a different intern. \
                 Do not re-run this intern with the same task."
            );
        }

        let mut truth = String::new();
        truth.push_str("\n\n--- Ground truth (recorded by stynx, not the intern) ---\n");
        truth.push_str(&format!("  read:   {} files {}\n", act.read_paths.len(), dedupe_compact(&act.read_paths)));
        truth.push_str(&format!("  edited: {} files {}\n", act.edited_paths.len(), dedupe_compact(&act.edited_paths)));
        truth.push_str(&format!("  wrote:  {} files {}\n", act.written_paths.len(), dedupe_compact(&act.written_paths)));
        truth.push_str(&format!("  bash:   {} commands\n", act.bash_commands.len()));
        truth.push_str(&format!("  errors: {}\n", act.error_count));

        // Only flag the precise, verifiable problem: a claimed file change that
        // never actually happened. Everything else is trusted as-is.
        if out.contains("Files changed:") {
            let claimed_files = extract_claimed_files(&out);
            let real_changed: std::collections::HashSet<&str> = act.edited_paths.iter()
                .chain(act.written_paths.iter())
                .map(|s| s.as_str())
                .collect();
            let fake: Vec<&String> = claimed_files.iter()
                .filter(|p| !real_changed.contains(p.as_str()))
                .collect();
            if !fake.is_empty() {
                truth.push_str("\n[UNVERIFIED] these paths were listed under 'Files changed' but no edit/write was recorded — verify before trusting:\n");
                for f in fake {
                    truth.push_str(&format!("  • {f}\n"));
                }
            }
        }
        out.push_str(&truth);

        Ok(out)
    }
}

fn summarize_action(name: &str, input_json: &str, output: &str, is_error: bool) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_str(input_json).ok();
    let get_str = |k: &str| -> String {
        parsed
            .as_ref()
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let head = match name {
        "bash" => format!("bash $ {}", first_line_trunc(&get_str("command"), 100)),
        "read" => format!("read {}", get_str("file_path")),
        "file_write" => format!("write {}", get_str("file_path")),
        "file_edit" => format!("edit {}", get_str("file_path")),
        "glob" => format!("glob {}", get_str("pattern")),
        "grep" => {
            let p = get_str("pattern");
            let path = get_str("path");
            if path.is_empty() { format!("grep {p}") } else { format!("grep {p} in {path}") }
        }
        other => format!("{other}"),
    };
    let tail = if is_error {
        format!(" — ERROR: {}", first_line_trunc(output, 120))
    } else if name == "bash" || name == "file_write" || name == "file_edit" {
        let line_count = output.lines().filter(|l| !l.trim().is_empty()).count();
        if line_count > 0 { format!(" ({line_count} output lines)") } else { String::new() }
    } else {
        String::new()
    };
    format!("{head}{tail}")
}

fn dedupe_compact(paths: &[String]) -> String {
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut uniq: Vec<&str> = Vec::new();
    for p in paths {
        if seen.insert(p.as_str()) {
            uniq.push(p.as_str());
        }
    }
    if uniq.is_empty() { return String::new(); }
    let preview: Vec<String> = uniq.iter().take(8).map(|s| s.to_string()).collect();
    let more = if uniq.len() > 8 { format!(" (+{} more)", uniq.len() - 8) } else { String::new() };
    format!("[{}{}]", preview.join(", "), more)
}

fn extract_claimed_files(out: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_section = false;
    for line in out.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Files changed:") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.starts_with("Output:") || trimmed.starts_with("Summary:") || trimmed.starts_with("Actions taken:") {
            break;
        }
        if let Some(p) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("• ")) {
            let p = p.trim();
            if p.is_empty() || p.eq_ignore_ascii_case("none") {
                continue;
            }
            let p = p.trim_matches(|c: char| c == '`' || c == '"' || c == '\'');
            if p.starts_with('/') || p.contains('/') {
                files.push(p.to_string());
            }
        }
    }
    files
}

fn first_line_trunc(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or("").trim();
    if line.chars().count() <= max {
        return line.to_string();
    }
    let mut out: String = line.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
