use std::path::Path;

use claude_rust_provider::AnthropicProvider;
use claude_rust_types::PermissionMode;

use crate::infrastructure::command_types::CommandAction;
use crate::infrastructure::skills::Skill;
use crate::infrastructure::terminal::{BOLD, CYAN, DIM, RED, RESET, select_from_list};

pub fn handle_memory(cwd: &str) -> String {
    let names = ["CLAUDE.md", "MEMORY.md"];
    let mut found = false;
    let mut out = String::new();
    for name in &names {
        for dir in [cwd, &format!("{cwd}/.claude")] {
            let path = format!("{dir}/{name}");
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push_str(&format!("\n  {BOLD}{CYAN}{path}{RESET}\n\n"));
                for line in text.lines() { out.push_str(&format!("  {DIM}{line}{RESET}\n")); }
                found = true;
            }
        }
    }
    if !found { out = format!("\n  {DIM}No CLAUDE.md or MEMORY.md found in {cwd}{RESET}\n"); }
    out
}

pub fn handle_export(conversation: &claude_rust_types::Conversation, cwd: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let path = format!("{cwd}/conversation-{ts}.md");
    let mut md = String::from("# Conversation Export\n\n");
    for msg in &conversation.messages {
        let role = match msg.role {
            claude_rust_types::Role::User => "**User**",
            claude_rust_types::Role::Assistant => "**Assistant**",
        };
        md.push_str(&format!("## {role}\n\n"));
        for block in &msg.content {
            if let claude_rust_types::ContentBlock::Text { text } = block { md.push_str(text); md.push_str("\n\n"); }
        }
    }
    match std::fs::write(&path, &md) {
        Ok(_) => format!("\n  {DIM}Exported to {path}{RESET}\n"),
        Err(e) => format!("\n  {RED}Export failed: {e}{RESET}\n"),
    }
}

pub fn handle_rewind(conversation: &claude_rust_types::Conversation, system_prompt: &str, n: usize) -> claude_rust_types::Conversation {
    let mut msgs = conversation.messages.clone();
    let remove = (n * 2).min(msgs.len());
    msgs.truncate(msgs.len() - remove);
    let mut new_conv = claude_rust_types::Conversation { messages: msgs, ..Default::default() };
    new_conv.system = Some(system_prompt.to_string());
    new_conv
}

pub fn git_diff(cwd: &str, args: &[&str]) -> String {
    std::process::Command::new("git").args(args).current_dir(cwd).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

pub fn handle_review_prompt(cwd: &str) -> Option<String> {
    let diff = git_diff(cwd, &["diff", "HEAD"]);
    if diff.is_empty() { return None; }
    Some(format!("Please review these git changes and provide feedback on code quality, potential bugs, and improvements:\n\n```diff\n{diff}\n```"))
}

pub fn handle_commit_prompt(cwd: &str) -> Option<String> {
    let staged = git_diff(cwd, &["diff", "--staged"]);
    let diff = if staged.is_empty() { git_diff(cwd, &["diff", "HEAD"]) } else { staged };
    if diff.is_empty() { return None; }
    Some(format!("Generate a concise git commit message for these changes. Output only the commit message, no explanation:\n\n```diff\n{diff}\n```"))
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
            if !path.is_empty() && Path::new(path).exists()
                && let Ok(content) = std::fs::read_to_string(path) {
                    appended.push_str(&format!("\n\n<file path=\"{path}\">{content}</file>"));
                    for _ in 0..raw.len() { chars.next(); }
                    continue;
                }
        }
        result.push(c);
    }
    result.push_str(&appended);
    result
}

pub fn try_skill(input: &str, skills: &[Skill]) -> Option<CommandAction> {
    let without_slash = input.strip_prefix('/')?;
    let (cmd_name, args) = without_slash
        .split_once(char::is_whitespace).map(|(n, a)| (n, a.trim()))
        .unwrap_or((without_slash, ""));
    let skill = skills.iter().find(|s| s.name == cmd_name)?;
    let prompt = skill.expand(args);
    if prompt.is_empty() {
        let hint = skill.argument_hint.as_deref().unwrap_or("...");
        return Some(CommandAction::Output(format!("\n  {RED}Usage: /{cmd_name} {hint}{RESET}\n")));
    }
    Some(CommandAction::SendToEngine(prompt, skill.allowed_tools.clone()))
}

pub fn handle_model_cmd(name: &str, provider: &AnthropicProvider, mode_flag: &std::sync::Arc<std::sync::atomic::AtomicU8>) -> Option<CommandAction> {
    if name.is_empty() {
        let current = provider.model_name();
        let items: Vec<(String, String)> = vec![
            ("claude-sonnet-4-6".into(), "claude-sonnet-4-6".into()),
            ("claude-opus-4-6".into(), "claude-opus-4-6".into()),
            ("claude-haiku-4-5-20251001".into(), "claude-haiku-4-5-20251001".into()),
            ("claude-sonnet-4-5-20250929".into(), "claude-sonnet-4-5-20250929".into()),
        ];
        match select_from_list("Models", &items, &current) {
            Some(selected) => {
                provider.set_model(&selected);
                Some(CommandAction::Output(format!("\n  {DIM}Model →{RESET} {BOLD}{CYAN}{selected}{RESET}\n")))
            }
            None => Some(CommandAction::Continue),
        }
    } else {
        let resolved = match name {
            "opus" => "claude-opus-4-6",
            "sonnet" => "claude-sonnet-4-6",
            "haiku" => "claude-haiku-4-5-20251001",
            other => other,
        };
        let _ = mode_flag;
        provider.set_model(resolved);
        Some(CommandAction::Output(format!("\n  {DIM}Model →{RESET} {BOLD}{CYAN}{resolved}{RESET}\n")))
    }
}

pub fn handle_effort_cmd(level: &str, provider: &AnthropicProvider) -> CommandAction {
    if level.is_empty() {
        let current = provider.get_effort().unwrap_or_else(|| "auto".to_string());
        return CommandAction::Output(format!(
            "\n  {BOLD}{CYAN}Effort Levels{RESET}  {DIM}(current: {current}){RESET}\n\n\
             {DIM}  low{RESET}     Quick, concise responses\n\
             {DIM}  medium{RESET}  Balanced (default)\n\
             {DIM}  high{RESET}    Thorough, detailed responses\n\
             {DIM}  max{RESET}     Maximum depth and analysis\n\
             {DIM}  auto{RESET}    Clear effort override\n"
        ));
    }
    if level == "auto" {
        provider.clear_effort();
        return CommandAction::Output(format!("\n  {DIM}Effort → auto (cleared){RESET}\n"));
    }
    if !["low", "medium", "high", "max"].contains(&level) {
        return CommandAction::Output(format!("\n  {DIM}Invalid effort level. Use: low, medium, high, max, auto{RESET}\n"));
    }
    provider.set_effort(level);
    if level == "max" {
        provider.set_max_tokens(64000);
    }
    CommandAction::Output(format!("\n  {DIM}Effort →{RESET} {BOLD}{CYAN}{level}{RESET}\n"))
}

pub fn handle_plan_task(task: &str, mode_flag: &std::sync::Arc<std::sync::atomic::AtomicU8>) -> CommandAction {
    PermissionMode::Plan.store(mode_flag);
    let prompt = format!(
        "You are now in plan mode. Use read-only tools to explore the codebase, then write a detailed step-by-step plan for this task:\n\n{task}\n\nPresent the complete plan as text, then call exit_plan_mode to submit it for review."
    );
    CommandAction::SendToEngine(prompt, vec![])
}
