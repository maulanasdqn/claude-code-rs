pub mod application;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
mod tests;

pub use application::{
    CommandRegistry, execute_command, expand_file_references, expand_message_content,
    local_handler, parse_command, prompt_handler, register_builtin_commands,
};
pub use domain::{
    CommandAvailability, CommandDefinition, CommandOutput, CommandResult, CommandSource,
    CommandType, InputPreprocessor, SlashCommand,
};
pub use infrastructure::FileReferenceExpander;
