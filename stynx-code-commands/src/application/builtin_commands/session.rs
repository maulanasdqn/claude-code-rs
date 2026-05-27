use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
