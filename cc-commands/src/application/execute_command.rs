use crate::domain::{CommandResult, SlashCommand};
use crate::infrastructure::handlers;

pub fn execute_command(command: SlashCommand) -> CommandResult {
    match command {
        SlashCommand::Help => handlers::handle_help(),
        SlashCommand::Clear => handlers::handle_clear(),
        SlashCommand::Compact => handlers::handle_compact(),
        SlashCommand::Model(name) => handlers::handle_model(&name),
        SlashCommand::Quit => CommandResult::Quit,
    }
}
