use crate::domain::SlashCommand;

pub fn parse_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();

    if trimmed == "/help" {
        return Some(SlashCommand::Help);
    }

    if trimmed == "/clear" {
        return Some(SlashCommand::Clear);
    }

    if trimmed == "/compact" {
        return Some(SlashCommand::Compact);
    }

    if trimmed == "/quit" || trimmed == "/exit" {
        return Some(SlashCommand::Quit);
    }

    if let Some(rest) = trimmed.strip_prefix("/model ") {
        let name = rest.trim();
        if !name.is_empty() {
            return Some(SlashCommand::Model(name.to_string()));
        }
    }

    None
}
