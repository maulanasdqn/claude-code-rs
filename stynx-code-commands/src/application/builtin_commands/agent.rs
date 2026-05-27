use crate::application::command_registry::{local_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
