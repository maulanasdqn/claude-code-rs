use crate::domain::CommandResult;

pub async fn handle_status() -> CommandResult {
    match tokio::process::Command::new("git")
        .args(["status", "--short"])
        .output()
        .await
    {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if stdout.is_empty() {
                CommandResult::Output("  \x1b[2mWorking tree clean.\x1b[0m".into())
            } else {
                let mut output = String::from("  \x1b[1mGit status:\x1b[0m\n");
                for line in stdout.lines() {
                    output.push_str(&format!("  \x1b[2m{line}\x1b[0m\n"));
                }
                CommandResult::Output(output)
            }
        }
        Err(e) => CommandResult::Output(format!("  \x1b[31m✗ git status failed: {e}\x1b[0m")),
    }
}
