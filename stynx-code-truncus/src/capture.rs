use stynx_code_types::{ContentBlock, Conversation};

use crate::client::ApiClient;
use crate::config::Config;
use crate::dto::{IngestRequest, Msg};
use crate::project::project_from_cwd;
use crate::util::{machine, now_ms, truncate};

const DEFAULT_INTERVAL_SECS: u64 = 300;

/// Capture the current conversation into Truncus memory. Best-effort and
/// non-blocking to the caller's outcome: unconfigured / empty / failed captures
/// are silently skipped or logged. Throttled per session (default 5 min, override
/// with `TRUNCUS_CAPTURE_INTERVAL_SECS`) unless `force` is set — pass `force` at
/// true end-of-session so the final state is always persisted.
pub async fn capture(session_id: Option<&str>, cwd: &str, conversation: &Conversation, force: bool) {
    let Some(cfg) = Config::load() else {
        return;
    };
    let session_id = match session_id {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => return,
    };
    let messages = conversation_to_msgs(conversation);
    if messages.is_empty() {
        return;
    }
    if !force && !due(&session_id) {
        return;
    }

    let now = now_ms();
    let started_at = session_id.parse::<i64>().ok().unwrap_or(now);
    let request = IngestRequest {
        session_id: session_id.clone(),
        project: project_from_cwd(cwd),
        cwd: cwd.to_string(),
        machine: machine(),
        started_at,
        ended_at: now,
        messages,
    };

    match ApiClient::new(&cfg).ingest(&request).await {
        Ok(resp) => {
            tracing::debug!("truncus captured session {} ({})", resp.id, resp.status);
            mark(&session_id);
        }
        Err(e) => tracing::warn!("truncus capture failed: {e}"),
    }
}

/// Flatten a `Conversation` into role/text messages for ingestion. Internal
/// reasoning (Thinking) is dropped; tool calls/results are compacted so the
/// payload stays reasonable while preserving what the work actually did.
fn conversation_to_msgs(conversation: &Conversation) -> Vec<Msg> {
    conversation
        .messages
        .iter()
        .filter_map(|message| {
            let mut parts: Vec<String> = Vec::new();
            for block in &message.content {
                match block {
                    ContentBlock::Text { text } if !text.trim().is_empty() => parts.push(text.clone()),
                    ContentBlock::ToolUse { name, input, .. } => {
                        parts.push(format!("[tool: {name}] {}", truncate(&input.to_string(), 500)));
                    }
                    ContentBlock::ToolResult { content, is_error, .. } => {
                        let tag = if is_error.unwrap_or(false) { "tool error" } else { "tool result" };
                        parts.push(format!("[{tag}] {}", truncate(content, 1000)));
                    }
                    ContentBlock::Image { .. } => parts.push("[image]".to_string()),
                    ContentBlock::Text { .. } | ContentBlock::Thinking { .. } => {}
                }
            }
            let text = parts.join("\n");
            if text.trim().is_empty() {
                None
            } else {
                Some(Msg {
                    role: message.role.to_string(),
                    text,
                })
            }
        })
        .collect()
}

fn interval_secs() -> u64 {
    std::env::var("TRUNCUS_CAPTURE_INTERVAL_SECS")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS)
}

fn state_path(session_id: &str) -> std::path::PathBuf {
    let safe: String = session_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    std::env::temp_dir().join(format!("stynx-truncus-capture-{safe}"))
}

fn due(session_id: &str) -> bool {
    std::fs::metadata(state_path(session_id))
        .and_then(|meta| meta.modified())
        .map(|modified| {
            modified
                .elapsed()
                .map(|age| age.as_secs() >= interval_secs())
                .unwrap_or(true)
        })
        .unwrap_or(true)
}

fn mark(session_id: &str) {
    let _ = std::fs::write(state_path(session_id), b"");
}

#[cfg(test)]
mod tests {
    use super::conversation_to_msgs;
    use serde_json::json;
    use stynx_code_types::{ContentBlock, Conversation, Message, Role};

    #[test]
    fn flattens_roles_tools_and_drops_thinking() {
        let conv = Conversation {
            system: Some("sys".into()),
            messages: vec![
                Message::user("fix the bug"),
                Message {
                    role: Role::Assistant,
                    content: vec![
                        ContentBlock::Thinking { thinking: "secret reasoning".into() },
                        ContentBlock::Text { text: "on it".into() },
                        ContentBlock::ToolUse {
                            id: "t1".into(),
                            name: "bash".into(),
                            input: json!({"command": "cargo test"}),
                        },
                    ],
                },
                Message {
                    role: Role::User,
                    content: vec![ContentBlock::ToolResult {
                        tool_use_id: "t1".into(),
                        content: "ok".into(),
                        is_error: Some(false),
                    }],
                },
            ],
        };

        let msgs = conversation_to_msgs(&conv);
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[0].text, "fix the bug");
        // Thinking dropped; text + tool call kept.
        assert!(!msgs[1].text.contains("secret reasoning"));
        assert!(msgs[1].text.contains("on it"));
        assert!(msgs[1].text.contains("[tool: bash]"));
        assert!(msgs[2].text.contains("[tool result]"));
    }

    #[test]
    fn empty_messages_are_skipped() {
        let conv = Conversation {
            system: None,
            messages: vec![Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Thinking { thinking: "only thinking".into() }],
            }],
        };
        assert!(conversation_to_msgs(&conv).is_empty());
    }
}
