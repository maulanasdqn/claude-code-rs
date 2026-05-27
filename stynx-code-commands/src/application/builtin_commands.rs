mod agent;
mod auth;
mod config;
mod context;
mod dev;
mod git;
mod ide;
mod misc;
mod remote;
mod session;

use crate::application::command_registry::CommandRegistry;

pub fn register_builtin_commands(registry: &mut CommandRegistry) {
    git::register(registry);
    context::register(registry);
    config::register(registry);
    session::register(registry);
    ide::register(registry);
    agent::register(registry);
    auth::register(registry);
    dev::register(registry);
    remote::register(registry);
    misc::register(registry);
}
