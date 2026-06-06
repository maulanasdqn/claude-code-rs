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

impl SubEngine {
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
        let mut out = String::new();
        if !act.text.trim().is_empty() {
            out.push_str(act.text.trim());
            out.push('\n');
        }
        if !act.actions.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str("Actions taken:\n");
            for a in &act.actions {
                out.push_str("  • ");
                out.push_str(a);
                out.push('\n');
            }
        }
        if out.trim().is_empty() {
            out.push_str(
                "[FAILED] intern produced ZERO output and took ZERO tool actions — it is broken, hung mid-thought, or refused the task. \
                 AUTO-RECOVER NOW (do not ask the user): take over yourself with bash/read/file_edit/file_write, or pick a different intern. \
                 Do not retry this intern with the same task."
            );
        }
        let missing_summary = !out.contains("Summary:");
        let missing_files = !out.contains("Files changed:");
        let no_tool_actions = !out.contains("Actions taken:");
        if missing_summary || missing_files || no_tool_actions {
            out.push_str("\n\n[MALFORMED] intern skipped required closing blocks (Summary / Files changed / Actions taken). \
            Treat any claims above as unverified. Verify by reading the files yourself before trusting the result.");
        }

        let mut truth = String::new();
        truth.push_str("\n\n--- Ground truth (recorded by stynx engine, NOT by the intern) ---\n");
        truth.push_str(&format!("  read:    {} files {}\n", act.read_paths.len(), dedupe_compact(&act.read_paths)));
        truth.push_str(&format!("  edited:  {} files {}\n", act.edited_paths.len(), dedupe_compact(&act.edited_paths)));
        truth.push_str(&format!("  wrote:   {} files {}\n", act.written_paths.len(), dedupe_compact(&act.written_paths)));
        truth.push_str(&format!("  bash:    {} commands\n", act.bash_commands.len()));
        truth.push_str(&format!("  errors:  {}\n", act.error_count));
        if act.edited_paths.is_empty() && act.written_paths.is_empty() && act.bash_commands.is_empty() {
            truth.push_str("\n[NO WRITES] intern modified ZERO files and ran ZERO commands. \
                If its reply claims any file change, that claim is FALSE. \
                Trust only the ground truth above.\n");
        }
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
                truth.push_str("\n[FALSE CLAIMS] intern's 'Files changed' list contains paths that were NEVER actually edited/written:\n");
                for f in fake {
                    truth.push_str(&format!("  ✗ {f}\n"));
                }
                truth.push_str("Treat the entire intern output as suspect. Verify the real changes via the ground-truth list above.\n");
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
