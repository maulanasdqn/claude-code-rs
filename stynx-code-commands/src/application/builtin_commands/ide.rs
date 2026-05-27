use crate::application::command_registry::{local_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
