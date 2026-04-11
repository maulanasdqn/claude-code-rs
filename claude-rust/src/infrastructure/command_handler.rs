use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_commands::{CommandResult, SlashCommand, execute_command, parse_command};
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, PermissionMode};

use crate::infrastructure::command_extras::{
    git_diff, handle_commit_prompt, handle_effort_cmd, handle_export, handle_memory,
    handle_model_cmd, handle_plan_task, handle_rewind, handle_review_prompt, try_skill,
};
use crate::infrastructure::command_types::CommandAction;
use crate::infrastructure::command_usage::render_usage;
use crate::infrastructure::skills::Skill;
use crate::infrastructure::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET, YELLOW};

const FAST_MODEL: &str = "claude-haiku-4-5-20251001";

#[allow(clippy::too_many_arguments)]
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

    if let Some(action) = try_skill(trimmed, skills) { return Some(action); }

    if let Some(task) = trimmed.strip_prefix("/plan ").map(str::trim).filter(|s| !s.is_empty()) {
        return Some(handle_plan_task(task, mode_flag));
    }

    let cmd = parse_command(input)?;

    if let SlashCommand::Model(ref name) = cmd {
        return handle_model_cmd(name, provider, mode_flag);
    }

    if matches!(cmd, SlashCommand::Fast) {
        let current = provider.model_name();
        if current.contains("haiku") {
            let restore = config.model.as_deref().unwrap_or("claude-sonnet-4-6");
            provider.set_model(restore);
            return Some(CommandAction::Output(format!("\n  {DIM}Fast mode off →{RESET} {BOLD}{CYAN}{restore}{RESET}\n")));
        } else {
            provider.set_model(FAST_MODEL);
            return Some(CommandAction::Output(format!("\n  {CYAN}{BOLD}⚡ Fast mode on →{RESET} {BOLD}{CYAN}{FAST_MODEL}{RESET}\n")));
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
            "\n  {color}{BOLD}{icon} {}{RESET} {DIM}— {}{RESET}\n", next.label(), next.description()
        )));
    }

    if matches!(cmd, SlashCommand::Config) {
        let json = serde_json::to_string_pretty(&config).unwrap_or_default();
        let result = claude_rust_commands::infrastructure::handlers::handle_config(&json);
        if let CommandResult::Output(text) = result { return Some(CommandAction::Output(format!("\n{text}\n"))); }
        return Some(CommandAction::Continue);
    }

    if matches!(cmd, SlashCommand::Permissions) {
        let result = claude_rust_commands::infrastructure::handlers::handle_permissions(
            &config.permissions.allow, &config.permissions.deny,
        );
        if let CommandResult::Output(text) = result { return Some(CommandAction::Output(format!("\n{text}\n"))); }
        return Some(CommandAction::Continue);
    }

    if matches!(cmd, SlashCommand::Think) {
        let enabled = provider.toggle_thinking();
        return Some(CommandAction::Output(if enabled {
            format!("\n  {CYAN}◆  Thinking enabled{RESET}  {DIM}— extended reasoning active{RESET}\n")
        } else {
            format!("\n  {DIM}◆  Thinking disabled{RESET}\n")
        }));
    }

    if matches!(cmd, SlashCommand::Usage) { return Some(CommandAction::Output(render_usage(provider).await)); }

    if let SlashCommand::Effort(ref level) = cmd { return Some(handle_effort_cmd(level, provider)); }

    if matches!(cmd, SlashCommand::Plan) {
        let current = PermissionMode::load(mode_flag);
        let next = if current == PermissionMode::Plan { PermissionMode::Normal } else { PermissionMode::Plan };
        next.store(mode_flag);
        return Some(CommandAction::Output(if next == PermissionMode::Plan {
            format!("\n  {MAGENTA}{BOLD}◆ Plan mode activated{RESET} {DIM}— only read-only tools available{RESET}\n")
        } else {
            format!("\n  {GREEN}{BOLD}✓ Plan mode deactivated{RESET} {DIM}— all tools available{RESET}\n")
        }));
    }

    if matches!(cmd, SlashCommand::Memory) { return Some(CommandAction::Output(handle_memory(cwd))); }
    if matches!(cmd, SlashCommand::Export) { return Some(CommandAction::Output(handle_export(conversation, cwd))); }
    if let SlashCommand::Rewind(n) = cmd { return Some(CommandAction::ReplaceConversation(handle_rewind(conversation, system_prompt, n))); }

    if matches!(cmd, SlashCommand::Review) {
        return Some(match handle_review_prompt(cwd) {
            Some(msg) => CommandAction::SendToEngine(msg, vec![]),
            None => CommandAction::Output(format!("\n  {DIM}No changes to review.{RESET}\n")),
        });
    }

    if matches!(cmd, SlashCommand::Commit) {
        return Some(match handle_commit_prompt(cwd) {
            Some(msg) => CommandAction::SendToEngine(msg, vec![]),
            None => CommandAction::Output(format!("\n  {DIM}No changes to commit.{RESET}\n")),
        });
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
