use crate::domain::SlashCommand;

pub fn parse_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();

    match trimmed {
        "/help" => return Some(SlashCommand::Help),
        "/clear" => return Some(SlashCommand::Clear),
        "/compact" => return Some(SlashCommand::Compact),
        "/cost" => return Some(SlashCommand::Cost),
        "/quit" | "/exit" => return Some(SlashCommand::Quit),
        "/diff" => return Some(SlashCommand::Diff),
        "/status" => return Some(SlashCommand::Status),
        "/doctor" => return Some(SlashCommand::Doctor),
        "/config" => return Some(SlashCommand::Config),
        "/permissions" => return Some(SlashCommand::Permissions),
        "/plan" => return Some(SlashCommand::Plan),
        "/mode" => return Some(SlashCommand::Mode),
        "/model" => return Some(SlashCommand::Model(String::new())),
        "/session" => return Some(SlashCommand::Session(String::new())),
        _ => {}
    }

    if let Some(rest) = trimmed.strip_prefix("/model ") {
        let name = rest.trim();
        if !name.is_empty() {
            return Some(SlashCommand::Model(name.to_string()));
        }
    }

    if let Some(rest) = trimmed.strip_prefix("/session ") {
        let id = rest.trim();
        if !id.is_empty() {
            return Some(SlashCommand::Session(id.to_string()));
        }
    }

    None
}
