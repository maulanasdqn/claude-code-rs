use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};

use claude_rust_commands::expand_message_content;
use claude_rust_engine::{QueryEngine, UndoStack, restore_undo};
use claude_rust_permission::ConfigAwarePermissionChecker;
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, Message, Role};
use tokio::sync::mpsc;

use super::command_extras::expand_at_mentions;
use super::command_handler::{CommandAction, handle_slash_command};
use super::conductor::CONDUCTOR_SYSTEM;
use super::event_renderer::{render_cost, render_error_box, render_exit_summary};
use super::input_history::{append_history, load_history};
use super::run_engine::run_engine;
use super::self_correct::run_self_correct;
use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET, clear_pasted_images, layout_width, print_banner, read_user_input, set_current_model, take_pasted_images};

#[allow(clippy::too_many_arguments)]
pub async fn run_loop(
    engine: Arc<QueryEngine>,
    conductor_engine: Arc<QueryEngine>,
    reflect_engine: Arc<QueryEngine>,
    session_repo: Arc<dyn claude_rust_memory::SessionRepository>,
    provider: Arc<AnthropicProvider>,
    config: claude_rust_config::Settings,
    mode_flag: Arc<AtomicU8>,
    cwd: String,
    system_prompt: String,
    mut conversation: Conversation,
    skills: Vec<Skill>,
    pause_flag: Arc<AtomicBool>,
    permission: Arc<ConfigAwarePermissionChecker>,
) {
    let session_start = std::time::Instant::now();
    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));
    let reflect_enabled = Arc::new(AtomicBool::new(false));
    let undo_stack = engine.undo_stack();
    let mut prompt_history: Vec<String> = load_history(&cwd);
    let mut pinned_files: Vec<String> = Vec::new();
    let model_id = provider.model_name();
    set_current_model(&model_id);
    let skill_names: Vec<String> = skills.iter()
        .filter(|s| s.user_invocable)
        .map(|s| s.name.clone())
        .collect();

    let (bg_tx, mut bg_rx) = mpsc::unbounded_channel::<Result<Conversation, claude_rust_errors::AppError>>();
    let mut bg_count: usize = 0;

    loop {
        let input = match read_user_input(&mode_flag, &prompt_history, &skill_names) {
            Some(s) if s.is_empty() => continue,
            Some(s) => s,
            None => break,
        };

        // Check for completed background tasks
        while let Ok(result) = bg_rx.try_recv() {
            match result {
                Ok(updated) => {
                    conversation = updated;
                    save_session(&session_repo, &conversation).await;
                    println!("  {CYAN}{BOLD}⌁{RESET}  {DIM}Background task completed{RESET}\n");
                }
                Err(e) if e.is_interrupted() => {
                    println!("  {DIM}⌁ Background task interrupted{RESET}\n");
                }
                Err(e) => {
                    render_error_box(&e.to_string());
                }
            }
        }

        // Handle special internal signals from shortcuts (prefixed with \x00)
        if let Some(signal) = input.strip_prefix('\x00') {
            clear_pasted_images();
            if let Some(bg_text) = signal.strip_prefix("background:") {
                let expanded = expand_with_pins(bg_text, &pinned_files);
                let mut bg_conv = conversation.clone();
                bg_conv.push(Message { role: Role::User, content: expand_message_content(&expanded) });

                bg_count += 1;
                let task_num = bg_count;
                println!("  {DIM}⌁ Task #{task_num} sent to background{RESET}\n");

                let bg_engine = engine.clone();
                let bg_total_input = total_input.clone();
                let bg_total_output = total_output.clone();
                let bg_mode_flag = mode_flag.clone();
                let bg_model_id = model_id.clone();
                let bg_cwd = cwd.clone();
                let bg_pause = pause_flag.clone();
                let bg_tx = bg_tx.clone();

                tokio::spawn(async move {
                    let result = run_engine(
                        &bg_engine, bg_conv,
                        &bg_total_input, &bg_total_output,
                        &bg_mode_flag, &bg_model_id, &bg_cwd, &bg_pause,
                    ).await;
                    let _ = bg_tx.send(result);
                });

                prompt_history.push(bg_text.to_string());
                append_history(bg_text, &cwd);
                continue;
            }
            match signal {
                "transcript" => { show_transcript(&conversation); continue; }
                "todos" => { show_todos(&cwd); continue; }
                _ => continue,
            }
        }

        if input.trim() == "/help" {
            print_help(&skills, &cwd);
            continue;
        }

        if let Some(task) = input.trim().strip_prefix("/conductor") {
            let task = task.trim();
            if task.is_empty() {
                println!("  {DIM}Usage: /conductor <task description>{RESET}\n");
            } else {
                let mut cond_conv = Conversation {
                    system: Some(CONDUCTOR_SYSTEM.to_string()),
                    ..Default::default()
                };
                cond_conv.push(Message { role: Role::User, content: expand_message_content(task) });
                match run_engine(&conductor_engine, cond_conv, &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
                    Ok(_) => {}
                    Err(e) if e.is_interrupted() => {}
                    Err(e) => render_error_box(&e.to_string()),
                }
            }
            continue;
        }

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

        if input.trim() == "/copy" {
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
                    .stdin(std::process::Stdio::piped())
                    .spawn();
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
            continue;
        }

        if input.trim() == "/login" {
            println!("  {DIM}Run {BOLD}claude login{RESET}{DIM} in your terminal to authenticate.{RESET}");
            println!("  {DIM}Credentials will be stored at ~/.claude/.credentials.json{RESET}\n");
            continue;
        }

        if input.trim() == "/logout" {
            let path = dirs_or_home().join(".claude").join(".credentials.json");
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    println!("  {DIM}✗ Failed to remove credentials: {e}{RESET}\n");
                } else {
                    println!("  {DIM}✓ Credentials removed. Restart to take effect.{RESET}\n");
                }
            } else {
                println!("  {DIM}No credentials file found.{RESET}\n");
            }
            continue;
        }

        if input.trim() == "/vim" {
            println!("  {DIM}Vim mode is always active. Use Esc to enter normal mode.{RESET}\n");
            continue;
        }

        if input.trim() == "/reflect" {
            let was = reflect_enabled.fetch_xor(true, Ordering::Relaxed);
            if was {
                println!("  {DIM}◆ Self-correction disabled{RESET}\n");
            } else {
                println!("  {CYAN}{BOLD}◆ Self-correction enabled{RESET}  {DIM}— responses will be auto-reviewed against system standards{RESET}\n");
            }
            continue;
        }

        if input.trim() == "/version" {
            println!("  {DIM}claude-rust v{}{RESET}\n", env!("CARGO_PKG_VERSION"));
            continue;
        }

        if input.starts_with('/') {
            match handle_slash_command(&input, &provider, &config, &mode_flag, &system_prompt, &cwd, &conversation, &skills).await {
                Some(CommandAction::Output(text)) => {
                    print!("{text}");
                    // Refresh model in status bar in case /model or /fast changed it
                    set_current_model(&provider.model_name());
                    continue;
                }
                Some(CommandAction::ReplaceConversation(c)) => {
                    let msg_count = c.messages.len();
                    conversation = c;
                    let current_model = provider.model_name();
                    set_current_model(&current_model);
                    print!("\x1b[2J\x1b[H");
                    print_banner(&cwd, &current_model);
                    println!("  {DIM}✓ {msg_count} messages kept.{RESET}\n");
                    continue;
                }
                Some(CommandAction::SendToEngine(msg, allowed_tools)) => {
                    if !allowed_tools.is_empty() {
                        permission.set_skill_allow_rules(allowed_tools);
                    }
                    let content = expand_message_content(&msg);
                    conversation.push(Message { role: Role::User, content });
                    match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
                        Ok(updated) => { conversation = updated; save_session(&session_repo, &conversation).await; }
                        Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
                        Err(e) => render_error_box(&e.to_string()),
                    }
                    permission.clear_skill_allow_rules();
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
        let mut content = expand_message_content(&expanded);

        // Attach any pasted images from Ctrl+V
        let images = take_pasted_images();
        if !images.is_empty() {
            let n = images.len();
            for (media_type, data) in images {
                content.push(claude_rust_types::ContentBlock::Image { media_type, data });
            }
            let s = if n == 1 { "" } else { "s" };
            println!("  {DIM}📎 {n} image{s} attached{RESET}");
        }

        conversation.push(Message { role: Role::User, content });

        match run_engine(&engine, conversation.clone(), &total_input, &total_output, &mode_flag, &model_id, &cwd, &pause_flag).await {
            Ok(mut updated) => {
                if reflect_enabled.load(Ordering::Relaxed) {
                    run_self_correct(&mut updated, &reflect_engine, &system_prompt).await;
                }
                conversation = updated;
                save_session(&session_repo, &conversation).await;
            }
            Err(e) if e.is_interrupted() => { conversation.messages.pop(); }
            Err(e) => render_error_box(&e.to_string()),
        }
    }

    save_session(&session_repo, &conversation).await;
    render_exit_summary(total_input.load(Ordering::Relaxed), total_output.load(Ordering::Relaxed), session_start.elapsed());
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

fn dirs_or_home() -> std::path::PathBuf {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

fn show_transcript(conversation: &Conversation) {
    use claude_rust_types::ContentBlock;

    if conversation.messages.is_empty() {
        println!("\n  {DIM}No messages in conversation.{RESET}\n");
        return;
    }

    let mut output = String::new();
    for msg in &conversation.messages {
        let role_label = match msg.role {
            Role::User => format!("{BOLD}{CYAN}You{RESET}"),
            Role::Assistant => format!("{BOLD}Assistant{RESET}"),
        };
        output.push_str(&format!("\n{role_label}\n"));
        for block in &msg.content {
            match block {
                ContentBlock::Text { text } => {
                    output.push_str(&format!("{text}\n"));
                }
                ContentBlock::ToolUse { name, .. } => {
                    output.push_str(&format!("{DIM}[tool: {name}]{RESET}\n"));
                }
                ContentBlock::ToolResult { content, .. } => {
                    let preview = if content.len() > 100 { &content[..100] } else { content };
                    output.push_str(&format!("{DIM}[result: {preview}...]{RESET}\n"));
                }
                ContentBlock::Thinking { thinking } => {
                    let preview = if thinking.len() > 80 { &thinking[..80] } else { thinking };
                    output.push_str(&format!("{DIM}[thinking: {preview}...]{RESET}\n"));
                }
                ContentBlock::Image { .. } => {
                    output.push_str(&format!("{DIM}[image]{RESET}\n"));
                }
            }
        }
        output.push_str(&format!("{DIM}────────────────────────────────{RESET}\n"));
    }

    // Try to pipe to pager, fallback to direct print
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less -R".into());
    let parts: Vec<&str> = pager.split_whitespace().collect();
    if let Some((bin, args)) = parts.split_first() {
        crossterm::terminal::disable_raw_mode().ok();
        if let Ok(mut child) = std::process::Command::new(bin)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                use std::io::Write;
                let _ = stdin.write_all(output.as_bytes());
            }
            let _ = child.wait();
            crossterm::terminal::enable_raw_mode().ok();
            return;
        }
        crossterm::terminal::enable_raw_mode().ok();
    }

    // Fallback: direct print
    print!("{output}");
}

fn show_todos(cwd: &str) {
    let todos_path = format!("{cwd}/.claude/todos.json");
    if !std::path::Path::new(&todos_path).exists() {
        println!("\n  {DIM}No todos found. Use the todo_write tool to create tasks.{RESET}\n");
        return;
    }

    match std::fs::read_to_string(&todos_path) {
        Ok(content) => {
            if let Ok(todos) = serde_json::from_str::<serde_json::Value>(&content) {
                println!("\n  {BOLD}{CYAN}Tasks{RESET}\n");
                if let Some(arr) = todos.as_array() {
                    if arr.is_empty() {
                        println!("  {DIM}No tasks.{RESET}\n");
                        return;
                    }
                    for (i, todo) in arr.iter().enumerate() {
                        let status = todo.get("status").and_then(|s| s.as_str()).unwrap_or("pending");
                        let content = todo.get("content").and_then(|s| s.as_str()).unwrap_or("(untitled)");
                        let icon = match status {
                            "completed" | "done" => "\x1b[32m✓\x1b[0m",
                            "in_progress" | "active" => "\x1b[33m●\x1b[0m",
                            _ => "\x1b[2m○\x1b[0m",
                        };
                        println!("  {icon} {}. {BOLD}{content}{RESET}", i + 1);
                    }
                    println!();
                } else if let Some(obj) = todos.as_object() {
                    if obj.is_empty() {
                        println!("  {DIM}No tasks.{RESET}\n");
                        return;
                    }
                    for (key, val) in obj {
                        let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("pending");
                        let icon = match status {
                            "completed" | "done" => "\x1b[32m✓\x1b[0m",
                            "in_progress" | "active" => "\x1b[33m●\x1b[0m",
                            _ => "\x1b[2m○\x1b[0m",
                        };
                        let content = val.get("content").and_then(|s| s.as_str()).unwrap_or(key.as_str());
                        println!("  {icon} {BOLD}{content}{RESET}");
                    }
                    println!();
                }
            } else {
                println!("\n  {DIM}Could not parse todos.json{RESET}\n");
            }
        }
        Err(_) => println!("\n  {DIM}Could not read todos file.{RESET}\n"),
    }
}

#[allow(clippy::too_many_arguments)]
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

fn print_help(skills: &[Skill], _cwd: &str) {
    let w = layout_width().min(72);

    println!();
    println!("  {BOLD}{CYAN}Slash Commands{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(w));

    let cmds: &[(&str, &str)] = &[
        ("/help",        "Show this help"),
        ("/clear",       "Clear conversation history"),
        ("/compact",     "Compact context to save tokens"),
        ("/cost",        "Show token usage and cost"),
        ("/usage",       "Show plan usage limits (OAuth only)"),
        ("/model [name]","Switch model  (aliases: opus/sonnet/haiku)"),
        ("/fast",        "Toggle fast mode (haiku)"),
        ("/think",       "Toggle extended thinking"),
        ("/effort",      "Set effort level (low/medium/high/max)"),
        ("/mode",        "Cycle permission mode"),
        ("/plan [task]", "Toggle plan mode, or plan a task"),
        ("/conductor <task>", "Orchestrate parallel worker agents"),
        ("/reflect",          "Toggle self-correction (default: off)"),
        ("/diff",        "Show git diff"),
        ("/status",      "Show git status"),
        ("/review",      "Review current git diff"),
        ("/commit",      "Generate a commit message"),
        ("/memory",      "Show CLAUDE.md"),
        ("/init",        "Create/update CLAUDE.md for this project"),
        ("/add <path>",  "Pin a file to every message"),
        ("/files",       "List pinned files"),
        ("/skills",      "List available skills"),
        ("/session",     "List or load previous sessions"),
        ("/rewind [n]",  "Remove last n exchanges (default 1)"),
        ("/export",      "Export conversation to markdown"),
        ("/copy",        "Copy last response to clipboard"),
        ("/undo [n]",    "Restore last n file edits"),
        ("/config",      "Show merged config"),
        ("/permissions", "Show allow/deny rules"),
        ("/version",     "Show version"),
        ("/vim",         "Vim mode status"),
        ("/login",       "Authentication instructions"),
        ("/logout",      "Clear stored credentials"),
        ("/quit",        "Exit session"),
    ];

    for (cmd, desc) in cmds {
        println!("  {CYAN}{cmd:<22}{RESET}  {DIM}{desc}{RESET}");
    }

    if !skills.is_empty() {
        println!();
        println!("  {BOLD}{CYAN}Skills{RESET}");
        println!("  {DIM}{}{RESET}", "─".repeat(w));
        for s in skills {
            if s.user_invocable {
                let hint = s.argument_hint.as_deref().map(|h| format!(" {h}")).unwrap_or_default();
                println!("  {CYAN}/{:<22}{RESET}  {DIM}{}{RESET}", format!("{}{hint}", s.name), s.description);
            }
        }
    }

    println!();
    println!("  {BOLD}{CYAN}Keyboard Shortcuts{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(w));

    let shortcuts: &[(&str, &str)] = &[
        ("Enter",       "Submit message"),
        ("Alt+Enter",   "Insert newline"),
        ("Shift+Tab",   "Cycle permission mode"),
        ("Ctrl+C / Esc","Interrupt current operation"),
        ("Ctrl+R",      "History search"),
        ("Ctrl+G",      "Open message in $EDITOR"),
        ("Ctrl+V",      "Paste image from clipboard"),
        ("Ctrl+B",      "Send to background task"),
        ("Ctrl+S",      "Stash / restore input buffer"),
        ("Ctrl+L",      "Clear screen"),
        ("Ctrl+U",      "Clear input line"),
        ("Ctrl+W",      "Delete previous word"),
        ("Ctrl+A / Home","Move to start of line"),
        ("Ctrl+E / End","Move to end of line"),
        ("Ctrl+O",      "Show transcript"),
        ("Ctrl+T",      "Show todos"),
        ("Alt+P",       "Model picker"),
        ("Alt+O",       "Toggle fast mode"),
        ("Alt+T",       "Toggle thinking"),
        ("Esc",         "Enter vim normal mode"),
        ("!<command>",  "Run shell command"),
    ];

    for (key, desc) in shortcuts {
        println!("  {CYAN}{key:<24}{RESET}  {DIM}{desc}{RESET}");
    }

    println!();
}
