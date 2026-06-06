use stynx_code_types::{Conversation, ContentBlock, Role};
use stynx_code_tui::DisplayMessage;

pub(super) fn conv_to_tui(conversation: &Conversation) -> Vec<DisplayMessage> {
    conversation.messages.iter().filter_map(|m| {
        let role = match m.role { Role::User => "user", Role::Assistant => "assistant" };
        let text = m.content.iter().filter_map(|b| {
            if let ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
        }).collect::<Vec<_>>().join("");
        if text.is_empty() { return None; }
        Some(DisplayMessage {
            role: role.to_string(),
            content: text,
            thinking: String::new(),
            tool_uses: Vec::new(),
            is_streaming: false,
        })
    }).collect()
}

pub(super) async fn export_transcript(conversation: &Conversation, cwd: &str) -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = format!("transcript-{ts}.md");
    let path = std::path::Path::new(cwd).join(&filename);

    let mut out = String::new();
    out.push_str(&format!("# stynx-code transcript — {ts}\n\n"));
    for msg in &conversation.messages {
        let header = match msg.role {
            Role::User => "## User",
            Role::Assistant => "## Assistant",
        };
        out.push_str(header);
        out.push_str("\n\n");
        for block in &msg.content {
            if let ContentBlock::Text { text } = block {
                out.push_str(text);
                out.push_str("\n\n");
            }
        }
    }

    tokio::fs::write(&path, out)
        .await
        .map_err(|e| format!("write failed: {e}"))?;
    Ok(path.display().to_string())
}

pub(super) fn fmt_elapsed(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    let ms = d.subsec_millis();
    if secs >= 60 {
        let m = secs / 60;
        let s = secs % 60;
        format!("{m}m {s}s")
    } else if secs >= 10 {
        format!("{secs}s")
    } else {
        format!("{secs}.{:01}s", ms / 100)
    }
}

pub(super) fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}k", n as f64 / 1_000.0) }
    else { n.to_string() }
}
