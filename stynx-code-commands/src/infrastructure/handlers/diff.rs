use crate::domain::CommandResult;

pub async fn handle_diff() -> CommandResult {
    let mut output = String::new();

    // Unstaged changes
    match tokio::process::Command::new("git")
        .args(["diff"])
        .output()
        .await
    {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if !stdout.is_empty() {
                output.push_str("  \x1b[1mUnstaged changes:\x1b[0m\n");
                for line in stdout.lines().take(50) {
                    output.push_str(&format!("  \x1b[2m{line}\x1b[0m\n"));
                }
                if stdout.lines().count() > 50 {
                    output.push_str("  \x1b[2m... (truncated)\x1b[0m\n");
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("  \x1b[31m✗ git diff failed: {e}\x1b[0m\n"));
        }
    }

    // Staged changes
    match tokio::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .await
    {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if !stdout.is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str("  \x1b[1mStaged changes:\x1b[0m\n");
                for line in stdout.lines().take(50) {
                    output.push_str(&format!("  \x1b[2m{line}\x1b[0m\n"));
                }
                if stdout.lines().count() > 50 {
                    output.push_str("  \x1b[2m... (truncated)\x1b[0m\n");
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("  \x1b[31m✗ git diff --cached failed: {e}\x1b[0m\n"));
        }
    }

    if output.is_empty() {
        output = "  \x1b[2mNo changes detected.\x1b[0m".into();
    }

    CommandResult::Output(output)
}
