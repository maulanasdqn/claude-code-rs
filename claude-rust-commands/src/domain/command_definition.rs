use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use claude_rust_errors::AppResult;

/// How the command executes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    /// Sends a prompt to the engine (e.g., /review, /commit).
    Prompt,
    /// Executes locally, returns a string result (e.g., /help, /status).
    Local,
    /// Executes locally, renders UI (maps to TUI widget in Phase 6).
    Interactive,
}

/// Where the command was defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSource {
    Builtin,
    Skill,
    Plugin,
    Mcp,
    Bundled,
}

/// Which auth contexts the command is available in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAvailability {
    ClaudeAi,
    Console,
    Universal,
}

/// The result of executing a command via the registry.
#[derive(Debug)]
pub enum CommandOutput {
    /// Display text to the user.
    Text(String),
    /// Inject a prompt into the engine.
    Prompt(String),
    /// Quit the application.
    Quit,
    /// No output (side effect only).
    None,
}

/// Handler function type for registry commands.
pub type CommandHandler = Arc<
    dyn Fn(&str) -> Pin<Box<dyn Future<Output = AppResult<CommandOutput>> + Send>>
        + Send
        + Sync,
>;

/// A dynamically registered command definition.
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
