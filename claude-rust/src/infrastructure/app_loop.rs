use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};

use claude_rust_commands::expand_message_content;
use claude_rust_engine::{QueryEngine, UndoStack, restore_undo};
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};

use super::command_extras::expand_at_mentions;
use super::command_handler::{CommandAction, handle_slash_command};
use super::event_renderer::{render_cost, render_error_box, render_exit_summary};
use super::input_history::{append_history, load_history};
use super::run_engine::run_engine;
use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET, print_banner, read_user_input};

pub async fn run_loop(
    engine: Arc<QueryEngine>,
    session_repo: Arc<dyn claude_rust_memory::SessionRepository>,
    provider: Arc<AnthropicProvider>,
    config: claude_rust_config::Settings,
    mode_flag: Arc<AtomicU8>,
    cwd: String,
    system_prompt: String,
    mut conversation: Conversation,
    skills: Vec<Skill>,
    pause_flag: Arc<AtomicBool>,
) {
    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let undo_stack = engine.undo_stack();
    let mut prompt_history: Vec<String> = load_history(&cwd);
    let mut pinned_files: Vec<String> = Vec::new();
    let model_id = provider.model_name();
    let skill_names: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();

    loop {
        let input = match read_user_input(&mode_flag, &prompt_history, &skill_names) {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        if input.trim() == "/cost" { render_cost(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed)); continue; }

        if let Some(shell_cmd) = input.trim().strip_prefix('!') {
            run_shell(shell_cmd);
            continue;
        }

        if input.trim() == "/init" {
            handle_init(&engine, &mut conversation, &session_repo, &total_input, &total_output, &mode_flag, &cwd, &model_id, &pause_flag).await;
            continue;
        }

        if let Some(rest) = input.trim().strip_prefix("/undo") {
            print!("{}", do_undo(&undo_stack, rest.trim().parse().unwrap_or(1)).await);
            continue;
        }

        if let Some(path) = input.trim().strip_prefix("/add ") {
            let path = path.trim().to_string();
            if std::path::Path::new(&path).exists() { println!("  {DIM}✓ Added {path} to context{RESET}\n"); pinned_files.push(path); }
            else { println!("  {DIM}✗ File not found: {path}{RESET}\n"); }
            continue;
        }

        if input.trim() == "/skills" {
            if skills.is_empty() {
                println!("  {DIM}No skills loaded. Add .md files to ~/.claude/skills/ or .claude/skills/{RESET}\n");
            } else {
                println!("  {BOLD}{CYAN}Skills{RESET}");
                for s in &skills {
                    let hint = s.argument_hint.as_deref().map(|h| format!(" {DIM}{h}{RESET}")).unwrap_or_default();
                    println!("  {DIM}/{}{RESET}{hint}  {DIM}{}{RESET}", s.name, s.description);
                }
                println!();
            }
            continue;
        }

        if input.trim() == "/files" {
            if pinned_files.is_empty() { println!("  {DIM}No files pinned. Use /add <path> to pin files.{RESET}\n"); }
            else { println!("  {DIM}Pinned files:{RESET}"); for f in &pinned_files { println!("  {DIM}  · {f}{RESET}"); } println!(); }
            continue;
        }

        if input.starts_with('/') {
            match handle_slash_command(&input, &provider, &config, &mode_flag, &system_prompt, &cwd, &conversation, &skills).await {
                Some(CommandAction::Output(text)) => { print!("{text}"); continue; }
                Some(CommandAction::ReplaceConversation(c)) => {
                    let msg_count = c.messages.len();
                    conversation = c;
                    print!("\x1b[2J\x1b[H");
                    print_banner(&cwd, &model_id);
                    println!("  {DIM}✓ {msg_count} messages kept.{RESET}\n");
                    continue;
                }
                Some(CommandAction::SendToEngine(msg)) => {
                    let content = expand_message_content(&msg);
                    conversation.push(Message { role: Role::User, content });
                    match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
                        Ok(updated) => { conversation = updated; save_session(&session_repo, &conversation).await; }
                        Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
                        Err(e) => render_error_box(&e.to_string()),
                    }
                    continue;
                }
                Some(CommandAction::Quit) => break,
                Some(CommandAction::Continue) => continue,
                None => {}
            }
        }

        prompt_history.push(input.clone());
        append_history(&input, &cwd);

        let expanded = expand_with_pins(&input, &pinned_files);
        conversation.push(Message { role: Role::User, content: expand_message_content(&expanded) });

        match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
            Ok(u) => { conversation = u; save_session(&session_repo, &conversation).await; }
            Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
            Err(e) => render_error_box(&e.to_string()),
        }
    }

    save_session(&session_repo, &conversation).await;
    render_exit_summary(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed));
}

fn run_shell(shell_cmd: &str) {
    let shell_cmd = shell_cmd.trim();
    match std::process::Command::new("sh").arg("-c").arg(shell_cmd).output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !stdout.is_empty() { print!("{stdout}"); }
            if !stderr.is_empty() { eprint!("{stderr}"); }
            if !out.status.success() {
                println!("  {DIM}exit {}{RESET}", out.status.code().unwrap_or(-1));
            }
        }
        Err(e) => println!("  {DIM}error: {e}{RESET}"),
    }
}

fn expand_with_pins(input: &str, pinned_files: &[String]) -> String {
    let mut s = expand_at_mentions(input);
    for path in pinned_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            s.push_str(&format!("\n\n<file path=\"{path}\">{content}</file>"));
        }
    }
    s
}

async fn do_undo(stack: &Arc<UndoStack>, n: usize) -> String {
    let results = restore_undo(stack, n).await;
    if results.is_empty() { return format!("\n  {DIM}Nothing to undo.{RESET}\n"); }
    let mut out = String::new();
    for (path, ok) in &results { out.push_str(&format!("\n  {DIM}{} Restored {path}{RESET}", if *ok { "✓" } else { "✗" })); }
    out.push('\n');
    out
}

async fn save_session(repo: &Arc<dyn claude_rust_memory::SessionRepository>, c: &Conversation) {
    if let Err(e) = claude_rust_memory::save_session(repo, c).await {
        tracing::warn!("failed to save session: {e}");
    }
}

async fn handle_init(
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
    let note = if std::path::Path::new(&format!("{cwd}/CLAUDE.md")).exists() { " (will update existing CLAUDE.md)" } else { "" };
    println!("  {DIM}Initializing CLAUDE.md{note}{RESET}\n");
    let msg = format!(
        "Analyze this project at `{cwd}` and create a CLAUDE.md file using the file_write tool. \
         First explore the project structure with glob and read key files. \
         The CLAUDE.md should contain: project overview (1-2 sentences), \
         essential commands (build, test, lint, format), code style rules, \
         and any important architectural patterns. Keep it concise (under 100 lines)."
    );
    let mut init_conv = conversation.clone();
    init_conv.push(Message { role: Role::User, content: expand_message_content(&msg) });
    match run_engine(engine, init_conv, total_input, total_output, mode_flag, model_id, cwd, pause_flag).await {
        Ok(updated) => { *conversation = updated; save_session(session_repo, conversation).await; }
        Err(e) if !e.is_interrupted() => render_error_box(&e.to_string()),
        _ => {}
    }
}
