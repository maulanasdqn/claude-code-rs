use crate::application::command_registry::{local_handler, prompt_handler, CommandRegistry};
use crate::domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandSource, CommandType,
};

pub(super) fn register(registry: &mut CommandRegistry) {
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
