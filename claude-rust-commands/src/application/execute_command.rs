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
        SlashCommand::Usage => CommandResult::Output("Use /usage from the main loop.".into()),
        SlashCommand::Mode => CommandResult::Output("Use /mode from the main loop.".into()),
        SlashCommand::Think => CommandResult::Output("Use /think from the main loop.".into()),
        SlashCommand::Add(_) => CommandResult::Output("Use /add from the main loop.".into()),
        SlashCommand::Files => CommandResult::Output("Use /files from the main loop.".into()),
        SlashCommand::Memory => CommandResult::Output("Use /memory from the main loop.".into()),
        SlashCommand::Export => CommandResult::Output("Use /export from the main loop.".into()),
        SlashCommand::Rewind(_) => CommandResult::Output("Use /rewind from the main loop.".into()),
        SlashCommand::Review => CommandResult::Output("Use /review from the main loop.".into()),
        SlashCommand::Commit => CommandResult::Output("Use /commit from the main loop.".into()),
        SlashCommand::Fast => CommandResult::Output("Use /fast from the main loop.".into()),
        SlashCommand::Quit => CommandResult::Quit,
    }
}
