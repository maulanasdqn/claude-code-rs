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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_help() {
        assert!(matches!(parse_command("/help"), Some(SlashCommand::Help)));
    }

    #[test]
    fn parse_clear() {
        assert!(matches!(parse_command("/clear"), Some(SlashCommand::Clear)));
    }

    #[test]
    fn parse_compact() {
        assert!(matches!(
            parse_command("/compact"),
            Some(SlashCommand::Compact)
        ));
    }

    #[test]
    fn parse_quit() {
        assert!(matches!(parse_command("/quit"), Some(SlashCommand::Quit)));
        assert!(matches!(parse_command("/exit"), Some(SlashCommand::Quit)));
    }

    #[test]
    fn parse_model() {
        match parse_command("/model claude-3-opus") {
            Some(SlashCommand::Model(name)) => assert_eq!(name, "claude-3-opus"),
            _ => panic!("expected Model variant"),
        }
    }

    #[test]
    fn parse_unknown() {
        assert!(parse_command("/unknown").is_none());
        assert!(parse_command("hello").is_none());
    }
}
