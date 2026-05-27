use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
