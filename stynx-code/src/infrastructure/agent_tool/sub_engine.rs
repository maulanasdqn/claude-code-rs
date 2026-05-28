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
}

impl SubEngine {
    pub(super) async fn run(&self, label: &str, system: &str, task: &str) -> AppResult<String> {
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
                        a.actions.push(summary);
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
            out.push_str("(intern returned no output and took no actions)");
        }
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

fn first_line_trunc(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or("").trim();
    if line.chars().count() <= max {
        return line.to_string();
    }
    let mut out: String = line.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}
