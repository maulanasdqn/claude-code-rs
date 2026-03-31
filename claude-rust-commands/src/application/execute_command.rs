use crate::domain::{CommandResult, SlashCommand};
use crate::infrastructure::handlers;

pub async fn execute_command(command: SlashCommand) -> CommandResult {
    match command {
        SlashCommand::Help => handlers::handle_help(),
        SlashCommand::Clear => handlers::handle_clear(),
        SlashCommand::Compact => handlers::handle_compact(),
        SlashCommand::Cost => CommandResult::Output("Use /cost from the main loop.".into()),
        SlashCommand::Model(name) => handlers::handle_model(&name),
        SlashCommand::Diff => handlers::handle_diff().await,
        SlashCommand::Status => handlers::handle_status().await,
        SlashCommand::Doctor => handlers::handle_doctor().await,
        SlashCommand::Config => CommandResult::Output("Use /config from the main loop.".into()),
        SlashCommand::Permissions => CommandResult::Output("Use /permissions from the main loop.".into()),
        SlashCommand::Session(id) => handlers::handle_session(&id).await,
        SlashCommand::Plan => CommandResult::Output("Use /plan from the main loop.".into()),
        SlashCommand::Quit => CommandResult::Quit,
    }
}
