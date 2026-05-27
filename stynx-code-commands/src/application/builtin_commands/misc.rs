use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
