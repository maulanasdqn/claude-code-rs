use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
                    .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
                Ok(CommandOutput::Text(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                ))
            } else {
                let output = std::process::Command::new("git")
                    .args(["checkout", args])
                    .output()
                    .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
                let text = if output.status.success() {
                    format!("Switched to branch '{args}'")
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };
                Ok(CommandOutput::Text(text))
            }
        }),
    });

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
                .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
            let text = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };
            Ok(CommandOutput::Text(text))
        }),
    });

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
                    .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
                Ok(CommandOutput::Text(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                ))
            } else {
                let output = std::process::Command::new("git")
                    .args(["tag", args])
                    .output()
                    .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
                let text = if output.status.success() {
                    format!("Created tag '{args}'")
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };
                Ok(CommandOutput::Text(text))
            }
        }),
    });

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
                .map_err(|e| stynx_code_errors::AppError::Tool(e.to_string()))?;
            let text = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };
            Ok(CommandOutput::Text(text))
        }),
    });
}
