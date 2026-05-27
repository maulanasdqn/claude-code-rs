use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use stynx_code_errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {

    Prompt,

    Local,

    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSource {
    Builtin,
    Skill,
    Plugin,
    Mcp,
    Bundled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAvailability {
    ClaudeAi,
    Console,
    Universal,
}

#[derive(Debug)]
pub enum CommandOutput {

    Text(String),

    Prompt(String),

    Quit,

    None,
}

pub type CommandHandler = Arc<
    dyn Fn(&str) -> Pin<Box<dyn Future<Output = AppResult<CommandOutput>> + Send>>
        + Send
        + Sync,
>;

pub struct CommandDefinition {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub command_type: CommandType,
    pub argument_hint: Option<String>,
    pub is_hidden: bool,
    pub availability: Vec<CommandAvailability>,
    pub source: CommandSource,
    pub handler: CommandHandler,
}

impl CommandDefinition {
    pub fn matches(&self, name: &str) -> bool {
        self.name == name || self.aliases.iter().any(|a| a == name)
    }
}

impl std::fmt::Debug for CommandDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandDefinition")
            .field("name", &self.name)
            .field("aliases", &self.aliases)
            .field("description", &self.description)
            .field("command_type", &self.command_type)
            .finish()
    }
}
