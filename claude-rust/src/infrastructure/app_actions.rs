use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

use claude_rust_engine::{QueryEngine, UndoStack, restore_undo};
use claude_rust_types::{Conversation, Message, Role};

use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET};

pub fn run_shell(shell_cmd: &str) {
    let shell_cmd = shell_cmd.trim();
    match std::process::Command::new("sh").arg("-c").arg(shell_cmd).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.is_empty() { print!("{stdout}"); }
            if !stderr.is_empty() { eprint!("{stderr}"); }
            if !out.status.success() { println!("  {DIM}exit {}{RESET}", out.status.code().unwrap_or(-1)); }
        }
        Err(e) => println!("  {DIM}error: {e}{RESET}"),
    }
}

pub fn expand_with_pins(input: &str, pinned_files: &[String]) -> String {
    let mut s = super::command_extras::expand_at_mentions(input);
    for path in pinned_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            s.push_str(&format!("\n\n<file path=\"{path}\">{content}</file>"));
        }
    }
    s
}

pub async fn do_undo(stack: &Arc<UndoStack>, n: usize) -> String {
    let results = restore_undo(stack, n).await;
    if results.is_empty() { return format!("\n  {DIM}Nothing to undo.{RESET}\n"); }
    let mut out = String::new();
    for (path, ok) in &results { out.push_str(&format!("\n  {DIM}{} Restored {path}{RESET}", if *ok { "✓" } else { "✗" })); }
    out.push('\n');
    out
}

pub async fn save_session(repo: &Arc<dyn claude_rust_memory::SessionRepository>, c: &Conversation) {
    if let Err(e) = claude_rust_memory::save_session(repo, c).await {
        tracing::warn!("failed to save session: {e}");
    }
}

pub fn dirs_or_home() -> std::path::PathBuf {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("."))
}

pub fn toggle_reflect(reflect_enabled: &Arc<AtomicBool>) -> bool {
    reflect_enabled.fetch_xor(true, Ordering::Relaxed)
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
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("xclip -selection clipboard 2>/dev/null || xsel --clipboard 2>/dev/null || pbcopy 2>/dev/null")
            .stdin(std::process::Stdio::piped()).spawn();
        match child {
            Ok(ref mut c) => {
                if let Some(ref mut stdin) = c.stdin { let _ = stdin.write_all(text.as_bytes()); }
                let _ = c.wait();
                println!("  {DIM}✓ Copied to clipboard{RESET}\n");
            }
            Err(_) => println!("  {DIM}✗ Install xclip or xsel for clipboard support{RESET}\n"),
        }
    } else {
        println!("  {DIM}No response to copy.{RESET}\n");
    }
}

pub fn handle_logout() {
    let path = dirs_or_home().join(".claude").join(".credentials.json");
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) { println!("  {DIM}✗ Failed to remove credentials: {e}{RESET}\n"); }
        else { println!("  {DIM}✓ Credentials removed. Restart to take effect.{RESET}\n"); }
    } else { println!("  {DIM}No credentials file found.{RESET}\n"); }
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_init(
    engine: &Arc<QueryEngine>,
    conversation: &mut Conversation,
    session_repo: &Arc<dyn claude_rust_memory::SessionRepository>,
    total_input: &Arc<AtomicU64>,
    total_output: &Arc<AtomicU64>,
    mode_flag: &Arc<AtomicU8>,
    cwd: &str,
    model_id: &str,
    pause_flag: &Arc<AtomicBool>,
) {
    use claude_rust_commands::expand_message_content;
    use super::run_engine::run_engine;
    use super::event_renderer::render_error_box;
    let note = if std::path::Path::new(&format!("{cwd}/CLAUDE.md")).exists() { " (will update existing CLAUDE.md)" } else { "" };
    println!("  {DIM}Initializing CLAUDE.md{note}{RESET}\n");
    let msg = format!("Analyze this project at `{cwd}` and create a CLAUDE.md file using the file_write tool. First explore the project structure with glob and read key files. The CLAUDE.md should contain: project overview (1-2 sentences), essential commands (build, test, lint, format), code style rules, and any important architectural patterns. Keep it concise (under 100 lines).");
    let mut init_conv = conversation.clone();
    init_conv.push(Message { role: Role::User, content: expand_message_content(&msg) });
    match run_engine(engine, init_conv, total_input, total_output, mode_flag, model_id, cwd, pause_flag).await {
        Ok(updated) => { *conversation = updated; save_session(session_repo, conversation).await; }
        Err(e) if !e.is_interrupted() => render_error_box(&e.to_string()),
        _ => {}
    }
}
