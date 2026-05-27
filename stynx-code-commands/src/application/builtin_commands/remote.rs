use crate::application::command_registry::{local_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
                "Welcome to stynx-code! Run /help to see available commands.".to_string(),
            ))
        }),
    });
}
