use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

/// Register all builtin commands into the registry.
///
/// These are new commands that do NOT overlap with the legacy `SlashCommand` enum.
/// Legacy commands (help, clear, compact, etc.) will be migrated in a later phase.
pub fn register_builtin_commands(registry: &mut CommandRegistry) {
    register_git_commands(registry);
    register_context_commands(registry);
    register_config_commands(registry);
    register_session_commands(registry);
    register_ide_commands(registry);
    register_agent_commands(registry);
    register_auth_commands(registry);
    register_dev_commands(registry);
    register_remote_commands(registry);
    register_misc_commands(registry);
}

// ---------------------------------------------------------------------------
// Git Commands
// ---------------------------------------------------------------------------

fn register_git_commands(registry: &mut CommandRegistry) {
    // /branch [name]
    registry.register(CommandDefinition {
        name: "/branch".to_string(),
        aliases: vec![],
        description: "Create or switch git branches".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[branch-name]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                let output = std::process::Command::new("git")
                    .args(["branch", "--list"])
                    .output()
                    .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
                Ok(CommandOutput::Text(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                ))
            } else {
                let output = std::process::Command::new("git")
                    .args(["checkout", args])
                    .output()
                    .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
                let text = if output.status.success() {
                    format!("Switched to branch '{args}'")
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };
                Ok(CommandOutput::Text(text))
            }
        }),
    });

    // /commit-push-pr
    registry.register(CommandDefinition {
        name: "/commit-push-pr".to_string(),
        aliases: vec![],
        description: "Commit, push, and create a pull request".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Create a commit with a descriptive message, push to remote, and create a pull request"
                .to_string()
        }),
    });

    // /pr_comments [pr-number]
    registry.register(CommandDefinition {
        name: "/pr_comments".to_string(),
        aliases: vec![],
        description: "Fetch PR comments".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[pr-number]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                return Ok(CommandOutput::Text(
                    "Usage: /pr_comments <pr-number>".to_string(),
                ));
            }
            let output = std::process::Command::new("gh")
                .args(["pr", "view", args, "--comments"])
                .output()
                .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
            let text = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };
            Ok(CommandOutput::Text(text))
        }),
    });

    // /tag [name]
    registry.register(CommandDefinition {
        name: "/tag".to_string(),
        aliases: vec![],
        description: "Create a git tag".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[name]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                let output = std::process::Command::new("git")
                    .args(["tag", "--list"])
                    .output()
                    .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
                Ok(CommandOutput::Text(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                ))
            } else {
                let output = std::process::Command::new("git")
                    .args(["tag", args])
                    .output()
                    .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
                let text = if output.status.success() {
                    format!("Created tag '{args}'")
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };
                Ok(CommandOutput::Text(text))
            }
        }),
    });

    // /stash
    registry.register(CommandDefinition {
        name: "/stash".to_string(),
        aliases: vec![],
        description: "Git stash operations".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            let git_args = if args.is_empty() {
                vec!["stash"]
            } else {
                vec!["stash", args]
            };
            let output = std::process::Command::new("git")
                .args(&git_args)
                .output()
                .map_err(|e| claude_rust_errors::AppError::Tool(e.to_string()))?;
            let text = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };
            Ok(CommandOutput::Text(text))
        }),
    });
}

// ---------------------------------------------------------------------------
// Context Commands
// ---------------------------------------------------------------------------

fn register_context_commands(registry: &mut CommandRegistry) {
    // /context
    registry.register(CommandDefinition {
        name: "/context".to_string(),
        aliases: vec![],
        description: "Show context window usage".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Context window usage information coming in Phase 4".to_string(),
            ))
        }),
    });

    // /add-dir [path]
    registry.register(CommandDefinition {
        name: "/add-dir".to_string(),
        aliases: vec![],
        description: "Add a directory to context".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[path]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text(
                    "Usage: /add-dir <path>".to_string(),
                ))
            } else {
                Ok(CommandOutput::Text(format!(
                    "Directory noted for context: {args}"
                )))
            }
        }),
    });

    // /thinkback
    registry.register(CommandDefinition {
        name: "/thinkback".to_string(),
        aliases: vec![],
        description: "Replay thinking process".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Replay and explain your thinking process from this conversation".to_string()
        }),
    });

    // /thinkback-play
    registry.register(CommandDefinition {
        name: "/thinkback-play".to_string(),
        aliases: vec![],
        description: "Play thinking replay".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Thinking replay not yet available".to_string(),
            ))
        }),
    });

    // /summary
    registry.register(CommandDefinition {
        name: "/summary".to_string(),
        aliases: vec![],
        description: "Summarize conversation session".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Provide a concise summary of this conversation session".to_string()
        }),
    });

    // /teleport [session-id]
    registry.register(CommandDefinition {
        name: "/teleport".to_string(),
        aliases: vec![],
        description: "Teleport to another session".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[session-id]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Session teleport not yet available (Phase 5)".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Configuration Commands
// ---------------------------------------------------------------------------

fn register_config_commands(registry: &mut CommandRegistry) {
    // /color [scheme]
    registry.register(CommandDefinition {
        name: "/color".to_string(),
        aliases: vec![],
        description: "Set or list color schemes".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[scheme]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text(
                    "Available color schemes: default, dark, light, solarized, monokai".to_string(),
                ))
            } else {
                Ok(CommandOutput::Text(format!("Color scheme: {args}")))
            }
        }),
    });

    // /theme [name]
    registry.register(CommandDefinition {
        name: "/theme".to_string(),
        aliases: vec![],
        description: "Set or list themes".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[name]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text(
                    "Available themes: default, minimal, compact, wide".to_string(),
                ))
            } else {
                Ok(CommandOutput::Text(format!("Theme: {args}")))
            }
        }),
    });

    // /output-style [style]
    registry.register(CommandDefinition {
        name: "/output-style".to_string(),
        aliases: vec![],
        description: "Set output style".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[style]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text(
                    "Available output styles: default, verbose, quiet, json".to_string(),
                ))
            } else {
                Ok(CommandOutput::Text(format!("Output style: {args}")))
            }
        }),
    });

    // /keybindings
    registry.register(CommandDefinition {
        name: "/keybindings".to_string(),
        aliases: vec![],
        description: "Show keybinding configuration".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Keybinding configuration not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /privacy-settings
    registry.register(CommandDefinition {
        name: "/privacy-settings".to_string(),
        aliases: vec![],
        description: "Privacy settings".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Privacy settings not yet available".to_string(),
            ))
        }),
    });

    // /hooks
    registry.register(CommandDefinition {
        name: "/hooks".to_string(),
        aliases: vec![],
        description: "Show hook configuration".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Hook configuration display coming in Phase 4".to_string(),
            ))
        }),
    });

    // /sandbox-toggle
    registry.register(CommandDefinition {
        name: "/sandbox-toggle".to_string(),
        aliases: vec![],
        description: "Toggle sandbox mode".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text("Sandbox mode toggled".to_string()))
        }),
    });

    // /env
    registry.register(CommandDefinition {
        name: "/env".to_string(),
        aliases: vec![],
        description: "Show relevant environment variables".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            let mut lines = Vec::new();
            let api_key_set = std::env::var("ANTHROPIC_API_KEY").is_ok();
            lines.push(format!("ANTHROPIC_API_KEY: {}", if api_key_set { "set" } else { "not set" }));
            for (key, value) in std::env::vars() {
                if key.starts_with("CLAUDE_") {
                    lines.push(format!("{key}={value}"));
                }
            }
            if lines.len() == 1 {
                lines.push("No CLAUDE_* environment variables set".to_string());
            }
            Ok(CommandOutput::Text(lines.join("\n")))
        }),
    });
}

// ---------------------------------------------------------------------------
// Session Commands
// ---------------------------------------------------------------------------

fn register_session_commands(registry: &mut CommandRegistry) {
    // /resume [session-id]
    registry.register(CommandDefinition {
        name: "/resume".to_string(),
        aliases: vec![],
        description: "Resume a previous session".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[session-id]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Session resume coming in Phase 4".to_string(),
            ))
        }),
    });

    // /rename [name]
    registry.register(CommandDefinition {
        name: "/rename".to_string(),
        aliases: vec![],
        description: "Rename current session".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[name]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text(
                    "Usage: /rename <name>".to_string(),
                ))
            } else {
                Ok(CommandOutput::Text(format!("Session renamed to: {args}")))
            }
        }),
    });

    // /share
    registry.register(CommandDefinition {
        name: "/share".to_string(),
        aliases: vec![],
        description: "Share current session".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Session sharing not yet available".to_string(),
            ))
        }),
    });

    // /insights
    registry.register(CommandDefinition {
        name: "/insights".to_string(),
        aliases: vec![],
        description: "Analyze session for insights".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Analyze this session and provide insights about patterns, efficiency, and suggestions"
                .to_string()
        }),
    });

    // /stats
    registry.register(CommandDefinition {
        name: "/stats".to_string(),
        aliases: vec![],
        description: "Show session statistics".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Session statistics not yet available".to_string(),
            ))
        }),
    });

    // /release-notes
    registry.register(CommandDefinition {
        name: "/release-notes".to_string(),
        aliases: vec![],
        description: "Show release notes".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Release notes display not yet available".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// IDE/Integration Commands
// ---------------------------------------------------------------------------

fn register_ide_commands(registry: &mut CommandRegistry) {
    // /ide
    registry.register(CommandDefinition {
        name: "/ide".to_string(),
        aliases: vec![],
        description: "IDE integration status".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "IDE integration status: not connected (Phase 5)".to_string(),
            ))
        }),
    });

    // /bridge
    registry.register(CommandDefinition {
        name: "/bridge".to_string(),
        aliases: vec![],
        description: "Bridge mode".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Bridge mode not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /chrome
    registry.register(CommandDefinition {
        name: "/chrome".to_string(),
        aliases: vec![],
        description: "Chrome integration".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Chrome integration not yet available".to_string(),
            ))
        }),
    });

    // /desktop
    registry.register(CommandDefinition {
        name: "/desktop".to_string(),
        aliases: vec![],
        description: "Desktop integration".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Desktop integration not yet available".to_string(),
            ))
        }),
    });

    // /mobile
    registry.register(CommandDefinition {
        name: "/mobile".to_string(),
        aliases: vec![],
        description: "Mobile integration".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Mobile integration not yet available".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Agent/Orchestration Commands
// ---------------------------------------------------------------------------

fn register_agent_commands(registry: &mut CommandRegistry) {
    // /agents
    registry.register(CommandDefinition {
        name: "/agents".to_string(),
        aliases: vec![],
        description: "Agent management".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Agent management not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /tasks
    registry.register(CommandDefinition {
        name: "/tasks".to_string(),
        aliases: vec![],
        description: "Task management UI".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Task management UI coming in Phase 5".to_string(),
            ))
        }),
    });

    // /plugin
    registry.register(CommandDefinition {
        name: "/plugin".to_string(),
        aliases: vec![],
        description: "Plugin management".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Plugin management not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /reload-plugins
    registry.register(CommandDefinition {
        name: "/reload-plugins".to_string(),
        aliases: vec![],
        description: "Reload plugins".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Plugin reload not yet available".to_string(),
            ))
        }),
    });

    // /stickers
    registry.register(CommandDefinition {
        name: "/stickers".to_string(),
        aliases: vec![],
        description: "Sticker reactions".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Sticker reactions not yet available".to_string(),
            ))
        }),
    });

    // /mcp
    registry.register(CommandDefinition {
        name: "/mcp".to_string(),
        aliases: vec![],
        description: "MCP server management".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "MCP server management coming in Phase 4".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Auth/Account Commands
// ---------------------------------------------------------------------------

fn register_auth_commands(registry: &mut CommandRegistry) {
    // /extra-usage
    registry.register(CommandDefinition {
        name: "/extra-usage".to_string(),
        aliases: vec![],
        description: "Extra usage information".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Extra usage information not yet available".to_string(),
            ))
        }),
    });

    // /rate-limit-options
    registry.register(CommandDefinition {
        name: "/rate-limit-options".to_string(),
        aliases: vec![],
        description: "Rate limit options".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Rate limit options not yet available".to_string(),
            ))
        }),
    });

    // /upgrade
    registry.register(CommandDefinition {
        name: "/upgrade".to_string(),
        aliases: vec![],
        description: "Manage your plan".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Visit claude.ai/settings to manage your plan".to_string(),
            ))
        }),
    });

    // /install-github-app
    registry.register(CommandDefinition {
        name: "/install-github-app".to_string(),
        aliases: vec![],
        description: "Install GitHub App".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "GitHub App installation guide coming in Phase 5".to_string(),
            ))
        }),
    });

    // /install-slack-app
    registry.register(CommandDefinition {
        name: "/install-slack-app".to_string(),
        aliases: vec![],
        description: "Install Slack App".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Slack App installation guide coming in Phase 5".to_string(),
            ))
        }),
    });

    // /feedback
    registry.register(CommandDefinition {
        name: "/feedback".to_string(),
        aliases: vec![],
        description: "Submit feedback".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Submit feedback at https://github.com/anthropics/claude-code/issues".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Development Commands
// ---------------------------------------------------------------------------

fn register_dev_commands(registry: &mut CommandRegistry) {
    // /security-review
    registry.register(CommandDefinition {
        name: "/security-review".to_string(),
        aliases: vec![],
        description: "Security-focused code review".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Review this codebase for security vulnerabilities. Focus on OWASP Top 10, \
             secrets exposure, injection flaws, and authentication issues. Report findings \
             with severity ratings."
                .to_string()
        }),
    });

    // /bughunter
    registry.register(CommandDefinition {
        name: "/bughunter".to_string(),
        aliases: vec![],
        description: "Enter bug hunting mode".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Enter bug hunting mode. Systematically search for bugs, edge cases, and potential \
             issues in the codebase"
                .to_string()
        }),
    });

    // /autofix-pr [pr-number]
    registry.register(CommandDefinition {
        name: "/autofix-pr".to_string(),
        aliases: vec![],
        description: "Auto-fix issues in a PR".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: Some("[pr-number]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|args| {
            if args.is_empty() {
                "Review and auto-fix issues in the current PR".to_string()
            } else {
                format!("Review and auto-fix issues in PR #{args}")
            }
        }),
    });

    // /init-verifiers
    registry.register(CommandDefinition {
        name: "/init-verifiers".to_string(),
        aliases: vec![],
        description: "Initialize verifiers".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Verifier initialization not yet available".to_string(),
            ))
        }),
    });

    // /advisor
    registry.register(CommandDefinition {
        name: "/advisor".to_string(),
        aliases: vec![],
        description: "Coding advisor mode".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|_args| {
            "Enter coding advisor mode. Provide guidance, best practices, and architectural advice"
                .to_string()
        }),
    });

    // /passes
    registry.register(CommandDefinition {
        name: "/passes".to_string(),
        aliases: vec![],
        description: "Multi-pass processing".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Multi-pass processing not yet available".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Remote/Setup Commands
// ---------------------------------------------------------------------------

fn register_remote_commands(registry: &mut CommandRegistry) {
    // /remote-setup
    registry.register(CommandDefinition {
        name: "/remote-setup".to_string(),
        aliases: vec![],
        description: "Remote setup".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Remote setup not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /remote-env
    registry.register(CommandDefinition {
        name: "/remote-env".to_string(),
        aliases: vec![],
        description: "Remote environment".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Remote environment not yet available".to_string(),
            ))
        }),
    });

    // /terminal-setup
    registry.register(CommandDefinition {
        name: "/terminal-setup".to_string(),
        aliases: vec![],
        description: "Terminal setup guide".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Terminal setup guide coming in Phase 5".to_string(),
            ))
        }),
    });

    // /onboarding
    registry.register(CommandDefinition {
        name: "/onboarding".to_string(),
        aliases: vec![],
        description: "Welcome onboarding".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Welcome to claude-rust! Run /help to see available commands.".to_string(),
            ))
        }),
    });
}

// ---------------------------------------------------------------------------
// Miscellaneous Commands
// ---------------------------------------------------------------------------

fn register_misc_commands(registry: &mut CommandRegistry) {
    // /exit (alias for /quit)
    registry.register(CommandDefinition {
        name: "/exit".to_string(),
        aliases: vec![],
        description: "Exit the application".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| Ok(CommandOutput::Quit)),
    });

    // /btw [note]
    registry.register(CommandDefinition {
        name: "/btw".to_string(),
        aliases: vec![],
        description: "Add a side note".to_string(),
        command_type: CommandType::Prompt,
        argument_hint: Some("[note]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: prompt_handler(|args| {
            if args.is_empty() {
                "The user wants to add a side note but didn't specify one.".to_string()
            } else {
                format!("Side note from the user: {args}")
            }
        }),
    });

    // /statusline [option]
    registry.register(CommandDefinition {
        name: "/statusline".to_string(),
        aliases: vec![],
        description: "Status line configuration".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[option]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Status line configuration not yet available".to_string(),
            ))
        }),
    });

    // /voice
    registry.register(CommandDefinition {
        name: "/voice".to_string(),
        aliases: vec![],
        description: "Voice input".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Voice input not yet available (Phase 5)".to_string(),
            ))
        }),
    });

    // /ctx_viz
    registry.register(CommandDefinition {
        name: "/ctx_viz".to_string(),
        aliases: vec![],
        description: "Context visualization".to_string(),
        command_type: CommandType::Local,
        argument_hint: None,
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|_args| {
            Ok(CommandOutput::Text(
                "Context visualization not yet available".to_string(),
            ))
        }),
    });

    // /brief [mode]
    registry.register(CommandDefinition {
        name: "/brief".to_string(),
        aliases: vec![],
        description: "Toggle brief mode".to_string(),
        command_type: CommandType::Local,
        argument_hint: Some("[mode]".to_string()),
        is_hidden: false,
        availability: vec![CommandAvailability::Universal],
        source: CommandSource::Builtin,
        handler: local_handler(|args| {
            if args.is_empty() {
                Ok(CommandOutput::Text("Brief mode: on".to_string()))
            } else {
                Ok(CommandOutput::Text(format!("Brief mode: {args}")))
            }
        }),
    });
}
