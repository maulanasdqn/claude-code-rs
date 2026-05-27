use crate::application::command_registry::{local_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
