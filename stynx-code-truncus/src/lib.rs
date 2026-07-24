//! Truncus memory integration for Stynx.
//!
//! Connects a Stynx session to the shared Truncus memory cluster (the same
//! Cloudflare Worker the `truncus` CLI/MCP use): it recalls distilled summaries
//! and lessons at session start, captures the conversation as it progresses,
//! and exposes on-demand memory search tools. Everything is best-effort — when
//! Truncus is not configured (`TRUNCUS_URL`/`TRUNCUS_TOKEN` or
//! `~/.config/truncus/config.toml`), all entry points are graceful no-ops.

mod capture;
mod client;
mod config;
mod dto;
mod project;
mod recall;
mod tools;
mod util;

pub use capture::capture;
pub use config::Config;
pub use project::project_from_cwd;
pub use recall::recall_section;
pub use tools::memory_tools;

/// Whether Truncus is configured on this machine.
pub fn is_configured() -> bool {
    Config::load().is_some()
}
