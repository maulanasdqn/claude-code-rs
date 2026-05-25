use stynx_code_errors::{AppError, AppResult};
use std::process::Command;

pub fn open_browser(url: &str) -> AppResult<()> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "start"
    } else {
        "xdg-open"
    };

    Command::new(cmd)
        .arg(url)
        .spawn()
        .map_err(|e| AppError::Provider(format!("failed to open browser with `{cmd}`: {e}")))?;

    Ok(())
}
