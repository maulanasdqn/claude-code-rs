use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_commands::{CommandResult, SlashCommand, execute_command, parse_command};
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, PermissionMode};

use crate::infrastructure::command_extras::{
    git_diff, handle_commit_prompt, handle_export, handle_memory, handle_rewind,
    handle_review_prompt,
};
use crate::infrastructure::skills::Skill;
use crate::infrastructure::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET, YELLOW};

const FAST_MODEL: &str = "claude-haiku-4-5-20251001";

pub enum CommandAction {
    Output(String),
    ReplaceConversation(Conversation),
    SendToEngine(String),
    Quit,
    Continue,
}

pub async fn handle_slash_command(
    input: &str,
    provider: &AnthropicProvider,
    config: &claude_rust_config::Settings,
    mode_flag: &Arc<AtomicU8>,
    system_prompt: &str,
    cwd: &str,
    conversation: &Conversation,
    skills: &[Skill],
) -> Option<CommandAction> {
    let trimmed = input.trim();

    if let Some(skill_action) = try_skill(trimmed, skills) {
        return Some(skill_action);
    }

    if let Some(task) = trimmed.strip_prefix("/plan ").map(str::trim).filter(|s| !s.is_empty()) {
        PermissionMode::Plan.store(mode_flag);
        let prompt = format!(
            "You are now in plan mode. Use read-only tools to explore the codebase, then write a detailed step-by-step plan for this task:\n\n{task}\n\nPresent the complete plan as text, then call exit_plan_mode to submit it for review."
        );
        return Some(CommandAction::SendToEngine(prompt));
    }

    let cmd = parse_command(input)?;

    if let SlashCommand::Model(ref name) = cmd {
        if name.is_empty() {
            return Some(CommandAction::Output(format!(
                "\n  {DIM}Current model:{RESET} {BOLD}{CYAN}{}{RESET}\n",
                provider.model_name()
            )));
        } else {
            provider.set_model(name);
            return Some(CommandAction::Output(format!(
                "\n  {DIM}Model →{RESET} {BOLD}{CYAN}{name}{RESET}\n"
            )));
        }
    }

    if matches!(cmd, SlashCommand::Fast) {
        let current = provider.model_name();
        if current.contains("haiku") {
            let restore = config.model.as_deref().unwrap_or("claude-sonnet-4-6");
            provider.set_model(restore);
            return Some(CommandAction::Output(format!(
                "\n  {DIM}Fast mode off →{RESET} {BOLD}{CYAN}{restore}{RESET}\n"
            )));
        } else {
            provider.set_model(FAST_MODEL);
            return Some(CommandAction::Output(format!(
                "\n  {CYAN}{BOLD}⚡ Fast mode on →{RESET} {BOLD}{CYAN}{FAST_MODEL}{RESET}\n"
            )));
        }
    }

    if matches!(cmd, SlashCommand::Mode) {
        let current = PermissionMode::load(mode_flag);
        let next = current.next();
        next.store(mode_flag);
        let (color, icon) = match next {
            PermissionMode::Normal => (GREEN, "●"),
            PermissionMode::AutoAccept => (YELLOW, "⚡"),
            PermissionMode::Plan => (MAGENTA, "◆"),
            PermissionMode::Bypass => (RED, "⚠"),
        };
        return Some(CommandAction::Output(format!(
            "\n  {color}{BOLD}{icon} {}{RESET} {DIM}— {}{RESET}\n",
            next.label(),
            next.description()
        )));
    }

    if matches!(cmd, SlashCommand::Config) {
        let json = serde_json::to_string_pretty(&config).unwrap_or_default();
        let result = claude_rust_commands::infrastructure::handlers::handle_config(&json);
        if let CommandResult::Output(text) = result {
            return Some(CommandAction::Output(format!("\n{text}\n")));
        }
        return Some(CommandAction::Continue);
    }

    if matches!(cmd, SlashCommand::Permissions) {
        let result = claude_rust_commands::infrastructure::handlers::handle_permissions(
            &config.permissions.allow,
            &config.permissions.deny,
        );
        if let CommandResult::Output(text) = result {
            return Some(CommandAction::Output(format!("\n{text}\n")));
        }
        return Some(CommandAction::Continue);
    }

    if matches!(cmd, SlashCommand::Think) {
        let enabled = provider.toggle_thinking();
        if enabled {
            return Some(CommandAction::Output(format!(
                "\n  {CYAN}◆  Thinking enabled{RESET}  {DIM}— extended reasoning active{RESET}\n"
            )));
        } else {
            return Some(CommandAction::Output(format!(
                "\n  {DIM}◆  Thinking disabled{RESET}\n"
            )));
        }
    }

    if matches!(cmd, SlashCommand::Plan) {
        let current = PermissionMode::load(mode_flag);
        let next = if current == PermissionMode::Plan { PermissionMode::Normal } else { PermissionMode::Plan };
        next.store(mode_flag);
        if next == PermissionMode::Plan {
            return Some(CommandAction::Output(format!(
                "\n  {MAGENTA}{BOLD}◆ Plan mode activated{RESET} {DIM}— only read-only tools available{RESET}\n"
            )));
        } else {
            return Some(CommandAction::Output(format!(
                "\n  {GREEN}{BOLD}✓ Plan mode deactivated{RESET} {DIM}— all tools available{RESET}\n"
            )));
        }
    }

    if matches!(cmd, SlashCommand::Memory) {
        return Some(CommandAction::Output(handle_memory(cwd)));
    }

    if matches!(cmd, SlashCommand::Export) {
        return Some(CommandAction::Output(handle_export(conversation, cwd)));
    }

    if let SlashCommand::Rewind(n) = cmd {
        return Some(CommandAction::ReplaceConversation(handle_rewind(conversation, system_prompt, n)));
    }

    if matches!(cmd, SlashCommand::Review) {
        match handle_review_prompt(cwd) {
            Some(msg) => return Some(CommandAction::SendToEngine(msg)),
            None => return Some(CommandAction::Output(format!("\n  {DIM}No changes to review.{RESET}\n"))),
        }
    }

    if matches!(cmd, SlashCommand::Commit) {
        match handle_commit_prompt(cwd) {
            Some(msg) => return Some(CommandAction::SendToEngine(msg)),
            None => return Some(CommandAction::Output(format!("\n  {DIM}No changes to commit.{RESET}\n"))),
        }
    }

    let _ = git_diff;

    match execute_command(cmd).await {
        CommandResult::Output(text) => Some(CommandAction::Output(format!("\n{text}\n"))),
        CommandResult::ReplaceConversation(mut c) => {
            c.system = Some(system_prompt.to_string());
            Some(CommandAction::ReplaceConversation(c))
        }
        CommandResult::Quit => Some(CommandAction::Quit),
    }
}

fn try_skill(input: &str, skills: &[Skill]) -> Option<CommandAction> {
    let without_slash = input.strip_prefix('/')?;
    let (cmd_name, args) = without_slash
        .split_once(char::is_whitespace)
        .map(|(n, a)| (n, a.trim()))
        .unwrap_or((without_slash, ""));
    let skill = skills.iter().find(|s| s.name == cmd_name)?;
    let prompt = skill.expand(args);
    if prompt.is_empty() {
        let hint = skill.argument_hint.as_deref().unwrap_or("...");
        return Some(CommandAction::Output(format!(
            "\n  {RED}Usage: /{cmd_name} {hint}{RESET}\n"
        )));
    }
    Some(CommandAction::SendToEngine(prompt))
}
