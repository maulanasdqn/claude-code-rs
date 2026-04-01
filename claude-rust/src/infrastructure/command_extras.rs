use std::path::Path;

use crate::infrastructure::terminal::{BOLD, CYAN, DIM, RED, RESET};

pub fn handle_memory(cwd: &str) -> String {
    let names = ["CLAUDE.md", "MEMORY.md"];
    let mut found = false;
    let mut out = String::new();
    for name in &names {
        for dir in [cwd, &format!("{cwd}/.claude")] {
            let path = format!("{dir}/{name}");
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push_str(&format!("\n  {BOLD}{CYAN}{path}{RESET}\n\n"));
                for line in text.lines() {
                    out.push_str(&format!("  {DIM}{line}{RESET}\n"));
                }
                found = true;
            }
        }
    }
    if !found {
        out = format!("\n  {DIM}No CLAUDE.md or MEMORY.md found in {cwd}{RESET}\n");
    }
    out
}

pub fn handle_export(conversation: &claude_rust_types::Conversation, cwd: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = format!("{cwd}/conversation-{ts}.md");
    let mut md = String::from("# Conversation Export\n\n");
    for msg in &conversation.messages {
        let role = match msg.role {
            claude_rust_types::Role::User => "**User**",
            claude_rust_types::Role::Assistant => "**Assistant**",
        };
        md.push_str(&format!("## {role}\n\n"));
        for block in &msg.content {
            if let claude_rust_types::ContentBlock::Text { text } = block {
                md.push_str(text);
                md.push_str("\n\n");
            }
        }
    }
    match std::fs::write(&path, &md) {
        Ok(_) => format!("\n  {DIM}Exported to {path}{RESET}\n"),
        Err(e) => format!("\n  {RED}Export failed: {e}{RESET}\n"),
    }
}

pub fn handle_rewind(
    conversation: &claude_rust_types::Conversation,
    system_prompt: &str,
    n: usize,
) -> claude_rust_types::Conversation {
    let mut msgs = conversation.messages.clone();
    let remove = (n * 2).min(msgs.len());
    msgs.truncate(msgs.len() - remove);
    let mut new_conv = claude_rust_types::Conversation { messages: msgs, ..Default::default() };
    new_conv.system = Some(system_prompt.to_string());
    new_conv
}

pub fn git_diff(cwd: &str, args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn handle_review_prompt(cwd: &str) -> Option<String> {
    let diff = git_diff(cwd, &["diff", "HEAD"]);
    if diff.is_empty() {
        return None;
    }
    Some(format!(
        "Please review these git changes and provide feedback on code quality, potential bugs, and improvements:\n\n```diff\n{diff}\n```"
    ))
}

pub fn handle_commit_prompt(cwd: &str) -> Option<String> {
    let staged = git_diff(cwd, &["diff", "--staged"]);
    let diff = if staged.is_empty() {
        git_diff(cwd, &["diff", "HEAD"])
    } else {
        staged
    };
    if diff.is_empty() {
        return None;
    }
    Some(format!(
        "Generate a concise git commit message for these changes. Output only the commit message, no explanation:\n\n```diff\n{diff}\n```"
    ))
}

pub fn expand_at_mentions(input: &str) -> String {
    let mut result = String::new();
    let mut appended = String::new();
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '@' {
            let start = i + 1;
            let rest = &input[start..];
            let end = rest.find(|c: char| c.is_whitespace() || c == ',').unwrap_or(rest.len());
            let raw = &rest[..end];
            let path = if raw.starts_with('"') && raw.ends_with('"') { &raw[1..raw.len()-1] } else { raw };
            if !path.is_empty() && Path::new(path).exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    appended.push_str(&format!("\n\n<file path=\"{path}\">{content}</file>"));
                    for _ in 0..raw.len() { chars.next(); }
                    continue;
                }
            }
        }
        result.push(c);
    }
    result.push_str(&appended);
    result
}
