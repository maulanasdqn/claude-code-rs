use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use claude_rust_commands::{CommandResult, SlashCommand, execute_command, parse_command};
use claude_rust_provider::AnthropicProvider;
use claude_rust_types::{Conversation, PermissionMode};

use crate::infrastructure::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, RESET, YELLOW};

pub enum CommandAction {
    Output(String),
    ReplaceConversation(Conversation),
    Quit,
    Continue,
}

pub async fn handle_slash_command(
    input: &str,
    provider: &AnthropicProvider,
    config: &claude_rust_config::Settings,
    mode_flag: &Arc<AtomicU8>,
    system_prompt: &str,
    _cwd: &str,
) -> Option<CommandAction> {
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

    if matches!(cmd, SlashCommand::Mode) {
        let current = PermissionMode::load(mode_flag);
        let next = current.next();
        next.store(mode_flag);
        let (color, icon) = match next {
            PermissionMode::Normal => (GREEN, "●"),
            PermissionMode::AutoAccept => (YELLOW, "⚡"),
            PermissionMode::Plan => (MAGENTA, "📋"),
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

    if matches!(cmd, SlashCommand::Plan) {
        let current = PermissionMode::load(mode_flag);
        let next = if current == PermissionMode::Plan {
            PermissionMode::Normal
        } else {
            PermissionMode::Plan
        };
        next.store(mode_flag);
        if next == PermissionMode::Plan {
            return Some(CommandAction::Output(format!(
                "\n  {MAGENTA}{BOLD}📋 Plan mode activated{RESET} {DIM}— only read-only tools available{RESET}\n"
            )));
        } else {
            return Some(CommandAction::Output(format!(
                "\n  {GREEN}{BOLD}✓ Plan mode deactivated{RESET} {DIM}— all tools available{RESET}\n"
            )));
        }
    }

    match execute_command(cmd).await {
        CommandResult::Output(text) => Some(CommandAction::Output(format!("\n{text}\n"))),
        CommandResult::ReplaceConversation(mut c) => {
            c.system = Some(system_prompt.to_string());
            Some(CommandAction::ReplaceConversation(c))
        }
        CommandResult::Quit => Some(CommandAction::Quit),
    }
}
