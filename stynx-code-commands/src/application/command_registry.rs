use std::collections::HashMap;

use stynx_code_errors::AppResult;

use crate::domain::{CommandDefinition, CommandHandler, CommandOutput, CommandType};
use crate::application::parse_command::parse_command;
use crate::application::execute_command::execute_command;
use crate::domain::command::CommandResult;

pub struct CommandRegistry {
    commands: Vec<CommandDefinition>,

    index: HashMap<String, usize>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn register(&mut self, def: CommandDefinition) {
        let idx = self.commands.len();
        self.index.insert(def.name.clone(), idx);
        for alias in &def.aliases {
            self.index.insert(alias.clone(), idx);
        }
        self.commands.push(def);
    }

    pub fn get(&self, name: &str) -> Option<&CommandDefinition> {
        self.index.get(name).map(|&i| &self.commands[i])
    }

    pub fn completions(&self, prefix: &str) -> Vec<String> {
        let mut results: Vec<String> = self.commands.iter()
            .filter(|cmd| !cmd.is_hidden)
            .flat_map(|cmd| {
                let mut names = vec![cmd.name.clone()];
                names.extend(cmd.aliases.iter().cloned());
                names
            })
            .filter(|name| name.starts_with(prefix))
            .collect();
        results.sort();
        results.dedup();
        results
    }

    pub fn list_commands(&self) -> Vec<(&str, &str, CommandType)> {
        self.commands.iter()
            .filter(|c| !c.is_hidden)
            .map(|c| (c.name.as_str(), c.description.as_str(), c.command_type))
            .collect()
    }

    pub async fn dispatch(&self, input: &str) -> Option<AppResult<CommandOutput>> {
        let trimmed = input.trim();

        let cmd_name = if let Some(space_idx) = trimmed.find(' ') {
            &trimmed[..space_idx]
        } else {
            trimmed
        };

        let args = trimmed.strip_prefix(cmd_name).unwrap_or("").trim();

        if let Some(def) = self.get(cmd_name) {
            return Some((def.handler)(args).await);
        }

        if let Some(cmd) = parse_command(trimmed) {
            let result = execute_command(cmd).await;
            return Some(Ok(match result {
                CommandResult::Output(text) => CommandOutput::Text(text),
                CommandResult::ReplaceConversation(_) => CommandOutput::None,
                CommandResult::Quit => CommandOutput::Quit,
            }));
        }

        None
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn local_handler<F>(f: F) -> CommandHandler
where
    F: Fn(&str) -> AppResult<CommandOutput> + Send + Sync + 'static,
{
    std::sync::Arc::new(move |args: &str| {
        let result = f(args);
        Box::pin(async move { result })
    })
}

pub fn prompt_handler<F>(f: F) -> CommandHandler
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    std::sync::Arc::new(move |args: &str| {
        let prompt = f(args);
        Box::pin(async move { Ok(CommandOutput::Prompt(prompt)) })
    })
}
