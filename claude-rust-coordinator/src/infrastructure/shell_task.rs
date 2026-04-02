use tokio::task::JoinHandle;

use claude_rust_errors::{AppError, AppResult};

pub fn spawn_shell_task(command: &str) -> JoinHandle<AppResult<String>> {
    let cmd = command.to_string();
    tokio::spawn(async move {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await
            .map_err(|e| AppError::Tool(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(stdout)
    })
}
