pub mod application;
pub mod domain;

pub use application::load_config;
pub use application::platform::{home_dir, shell_command};
pub use domain::{HookEntry, HooksConfig, PermissionSettings, Settings};
