use std::sync::Arc;

use claude_rust_types::{Conversation, Role};

use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET};

pub fn expand_with_pins(input: &str, pinned_files: &[String]) -> String {
    let mut s = super::command_extras::expand_at_mentions(input);
    for path in pinned_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            s.push_str(&format!("\n\n<file path=\"{path}\">{content}</file>"));
        }
    }
    s
}

pub async fn save_session(repo: &Arc<dyn claude_rust_memory::SessionRepository>, c: &Conversation) {
    if let Err(e) = claude_rust_memory::save_session(repo, c).await {
        tracing::warn!("failed to save session: {e}");
    }
}

pub fn show_skills(skills: &[Skill]) {
    if skills.is_empty() {
        println!("  {DIM}No skills loaded. Add .md files to ~/.claude/skills/ or .claude/skills/{RESET}\n");
    } else {
        println!("  {BOLD}{CYAN}Skills{RESET}");
        for s in skills {
            let hint = s.argument_hint.as_deref().map(|h| format!(" {DIM}{h}{RESET}")).unwrap_or_default();
            println!("  {DIM}/{}{RESET}{hint}  {DIM}{}{RESET}", s.name, s.description);
        }
        println!();
    }
}

pub fn show_files(pinned_files: &[String]) {
    if pinned_files.is_empty() { println!("  {DIM}No files pinned. Use /add <path> to pin files.{RESET}\n"); }
    else { println!("  {DIM}Pinned files:{RESET}"); for f in pinned_files { println!("  {DIM}  · {f}{RESET}"); } println!(); }
}

pub fn handle_add(path: &str, pinned_files: &mut Vec<String>) {
    let path = path.trim().to_string();
    if std::path::Path::new(&path).exists() {
        println!("  {DIM}✓ Added {path} to context{RESET}\n");
        pinned_files.push(path);
    } else {
        println!("  {DIM}✗ File not found: {path}{RESET}\n");
    }
}

pub fn copy_last_response(conversation: &Conversation) {
    use std::io::Write;
    let last_text = conversation.messages.iter().rev()
        .find(|m| m.role == Role::Assistant)
        .map(|m| m.content.iter().filter_map(|b| match b {
            claude_rust_types::ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        }).collect::<Vec<_>>().join("\n"));
    if let Some(text) = last_text {
        let mut child = if cfg!(windows) {
            std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", "Set-Clipboard -Value $input"])
                .stdin(std::process::Stdio::piped())
                .spawn()
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg("xclip -selection clipboard 2>/dev/null || xsel --clipboard 2>/dev/null || pbcopy 2>/dev/null")
                .stdin(std::process::Stdio::piped())
                .spawn()
        };
        match child {
            Ok(ref mut c) => {
                if let Some(ref mut stdin) = c.stdin { let _ = stdin.write_all(text.as_bytes()); }
                let _ = c.wait();
                println!("  {DIM}✓ Copied to clipboard{RESET}\n");
            }
            Err(_) => {
                if cfg!(windows) {
                    println!("  {DIM}✗ Failed to copy (powershell not available){RESET}\n");
                } else {
                    println!("  {DIM}✗ Install xclip or xsel for clipboard support{RESET}\n");
                }
            }
        }
    } else {
        println!("  {DIM}No response to copy.{RESET}\n");
    }
}
