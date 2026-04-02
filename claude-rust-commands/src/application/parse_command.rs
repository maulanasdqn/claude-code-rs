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
        "/usage" => return Some(SlashCommand::Usage),
        "/mode" => return Some(SlashCommand::Mode),
        "/model" => return Some(SlashCommand::Model(String::new())),
        "/session" => return Some(SlashCommand::Session(String::new())),
        "/think" => return Some(SlashCommand::Think),
        "/files" => return Some(SlashCommand::Files),
        "/memory" => return Some(SlashCommand::Memory),
        "/export" => return Some(SlashCommand::Export),
        "/review" => return Some(SlashCommand::Review),
        "/commit" => return Some(SlashCommand::Commit),
        "/fast" => return Some(SlashCommand::Fast),
        "/effort" => return Some(SlashCommand::Effort(String::new())),
        "/copy" => return Some(SlashCommand::Copy),
        "/login" => return Some(SlashCommand::Login),
        "/logout" => return Some(SlashCommand::Logout),
        "/vim" => return Some(SlashCommand::Vim),
        "/version" => return Some(SlashCommand::Version),
        "/rewind" => return Some(SlashCommand::Rewind(1)),
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

    if let Some(rest) = trimmed.strip_prefix("/add ") {
        let path = rest.trim();
        if !path.is_empty() {
            return Some(SlashCommand::Add(path.to_string()));
        }
    }

    if let Some(rest) = trimmed.strip_prefix("/rewind ") {
        let n: usize = rest.trim().parse().unwrap_or(1);
        return Some(SlashCommand::Rewind(n.max(1)));
    }

    if let Some(rest) = trimmed.strip_prefix("/effort ") {
        let level = rest.trim().to_lowercase();
        if !level.is_empty() {
            return Some(SlashCommand::Effort(level));
        }
    }

    None
}
