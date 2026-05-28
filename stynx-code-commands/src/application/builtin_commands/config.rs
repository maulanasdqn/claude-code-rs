use crate::application::command_registry::{local_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
