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
use crate::infrastructure::terminal::{BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET, YELLOW, select_from_list};

const FAST_MODEL: &str = "claude-haiku-4-5-20251001";

pub enum CommandAction {
    Output(String),
    ReplaceConversation(Conversation),
    /// Send a prompt to the engine, with optional skill-specific allowed tools.
    SendToEngine(String, Vec<String>),
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
        return Some(CommandAction::SendToEngine(prompt, vec![]));
    }

    let cmd = parse_command(input)?;

    if let SlashCommand::Model(ref name) = cmd {
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
                    return Some(CommandAction::Output(format!(
                        "\n  {DIM}Model →{RESET} {BOLD}{CYAN}{selected}{RESET}\n"
                    )));
                }
                None => return Some(CommandAction::Continue),
            }
        } else {
            // Resolve aliases when model name is provided directly
            let resolved = match name.as_str() {
                "opus" => "claude-opus-4-6",
                "sonnet" => "claude-sonnet-4-6",
                "haiku" => "claude-haiku-4-5-20251001",
                other => other,
            };

            provider.set_model(resolved);
            return Some(CommandAction::Output(format!(
                "\n  {DIM}Model →{RESET} {BOLD}{CYAN}{resolved}{RESET}\n"
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

    if matches!(cmd, SlashCommand::Usage) {
        return Some(CommandAction::Output(render_usage(provider).await));
    }

    if let SlashCommand::Effort(ref level) = cmd {
        if level.is_empty() {
            // Show current + options
            return Some(CommandAction::Output(format!(
                "\n  {BOLD}{CYAN}Effort Levels{RESET}\n\n\
                 {DIM}  low{RESET}     Quick, concise responses\n\
                 {DIM}  medium{RESET}  Balanced (default)\n\
                 {DIM}  high{RESET}    Thorough, detailed responses\n\
                 {DIM}  max{RESET}     Maximum depth and analysis\n"
            )));
        }
        let valid = ["low", "medium", "high", "max"];
        if !valid.contains(&level.as_str()) {
            return Some(CommandAction::Output(format!(
                "\n  {DIM}Invalid effort level. Use: low, medium, high, max{RESET}\n"
            )));
        }
        // For now just acknowledge - actual implementation needs provider changes
        return Some(CommandAction::Output(format!(
            "\n  {DIM}Effort →{RESET} {BOLD}{CYAN}{level}{RESET}\n"
        )));
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
            Some(msg) => return Some(CommandAction::SendToEngine(msg, vec![])),
            None => return Some(CommandAction::Output(format!("\n  {DIM}No changes to review.{RESET}\n"))),
        }
    }

    if matches!(cmd, SlashCommand::Commit) {
        match handle_commit_prompt(cwd) {
            Some(msg) => return Some(CommandAction::SendToEngine(msg, vec![])),
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
    Some(CommandAction::SendToEngine(prompt, skill.allowed_tools.clone()))
}

async fn render_usage(provider: &AnthropicProvider) -> String {
    if !provider.is_oauth() {
        return format!("\n  {DIM}/usage is only available for Claude AI subscribers (OAuth login){RESET}\n");
    }

    let utilization = match provider.fetch_usage().await {
        Ok(u) => u,
        Err(e) => return format!("\n  {BOLD}Error:{RESET} {e}\n"),
    };

    let mut out = String::new();
    out.push_str(&format!("\n  {BOLD}{CYAN}Plan Usage{RESET}\n\n"));

    fn render_limit(out: &mut String, title: &str, limit: &claude_rust_provider::RateLimit) {
        const DIM: &str = "\x1b[2m";
        const BOLD: &str = "\x1b[1m";
        const RESET: &str = "\x1b[0m";
        const CYAN: &str = "\x1b[36m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const RED: &str = "\x1b[31m";

        let pct = limit.utilization.unwrap_or(0.0);
        let filled = (pct / 100.0 * 30.0).round() as usize;
        let empty = 30usize.saturating_sub(filled);

        let bar_color = if pct >= 90.0 {
            RED
        } else if pct >= 70.0 {
            YELLOW
        } else {
            GREEN
        };

        let bar = format!(
            "{bar_color}{}{DIM}{}{RESET}",
            "█".repeat(filled),
            "░".repeat(empty)
        );

        out.push_str(&format!("  {BOLD}{title}{RESET}\n"));
        out.push_str(&format!("  {bar}  {CYAN}{pct:.0}%{RESET} used\n"));

        if let Some(ref resets) = limit.resets_at {
            out.push_str(&format!("  {DIM}Resets {resets}{RESET}\n"));
        }
        out.push('\n');
    }

    if let Some(ref l) = utilization.five_hour {
        render_limit(&mut out, "Session limit (5 hours)", l);
    }
    if let Some(ref l) = utilization.seven_day {
        render_limit(&mut out, "Weekly limit (7 days)", l);
    }
    if let Some(ref l) = utilization.seven_day_opus {
        render_limit(&mut out, "Opus limit (7 days)", l);
    }
    if let Some(ref l) = utilization.seven_day_sonnet {
        render_limit(&mut out, "Sonnet limit (7 days)", l);
    }

    if let Some(ref extra) = utilization.extra_usage {
        if extra.is_enabled {
            out.push_str(&format!("  {BOLD}Extra usage{RESET}\n"));
            if let (Some(used), Some(limit)) = (extra.used_credits, extra.monthly_limit) {
                out.push_str(&format!(
                    "  {DIM}Credits:{RESET} {CYAN}${used:.2}{RESET} / ${limit:.2}\n"
                ));
            }
            if let Some(pct) = extra.utilization {
                out.push_str(&format!("  {DIM}Utilization:{RESET} {CYAN}{pct:.0}%{RESET}\n"));
            }
            out.push('\n');
        }
    }

    if utilization.five_hour.is_none()
        && utilization.seven_day.is_none()
        && utilization.seven_day_opus.is_none()
        && utilization.seven_day_sonnet.is_none()
    {
        out.push_str(&format!("  {DIM}No usage data available.{RESET}\n"));
    }

    out
}
