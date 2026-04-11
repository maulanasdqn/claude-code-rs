use crate::domain::CommandResult;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub async fn handle_doctor() -> CommandResult {
    let mut output = format!("  {BOLD}Environment Diagnostics{RESET}\n\n");

    // Check git
    let git_ok = check_command("git", &["--version"]).await;
    output.push_str(&format_check("git", &git_ok));

    // Check ripgrep
    let rg_ok = check_command("rg", &["--version"]).await;
    output.push_str(&format_check("ripgrep (rg)", &rg_ok));

    // Check credential
    let cred_ok = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        Ok("API key set via ANTHROPIC_API_KEY".into())
    } else {
        // Try to check for OAuth credential
        match check_command("security", &["find-generic-password", "-s", "claude-code-credentials"]).await {
            Ok(_) => Ok("OAuth credential found in keychain".into()),
            Err(_) => Err("No ANTHROPIC_API_KEY or OAuth credential found".into()),
        }
    };
    output.push_str(&format_check("credentials", &cred_ok));

    // Check config files
    let home = claude_rust_config::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let global_config = home.join(".claude").join("settings.json");
    let project_config = std::env::current_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("settings.json");

    if global_config.exists() {
        output.push_str(&format!("  {GREEN}✓{RESET} Global config: {DIM}{}{RESET}\n", global_config.display()));
    } else {
        output.push_str(&format!("  {DIM}○ Global config: not found{RESET}\n"));
    }

    if project_config.exists() {
        output.push_str(&format!("  {GREEN}✓{RESET} Project config: {DIM}{}{RESET}\n", project_config.display()));
    } else {
        output.push_str(&format!("  {DIM}○ Project config: not found{RESET}\n"));
    }

    CommandResult::Output(output)
}

async fn check_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    match tokio::process::Command::new(cmd)
        .args(args)
        .output()
        .await
    {
        Ok(result) if result.status.success() => {
            let version = String::from_utf8_lossy(&result.stdout)
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            Ok(version)
        }
        Ok(result) => Err(String::from_utf8_lossy(&result.stderr).to_string()),
        Err(e) => Err(format!("not found: {e}")),
    }
}

fn format_check(name: &str, result: &Result<String, String>) -> String {
    match result {
        Ok(info) => format!("  {GREEN}✓{RESET} {name}: {DIM}{info}{RESET}\n"),
        Err(err) => format!("  {RED}✗{RESET} {name}: {DIM}{err}{RESET}\n"),
    }
}
