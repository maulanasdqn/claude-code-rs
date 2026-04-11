use crate::domain::CommandResult;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub async fn handle_session(arg: &str) -> CommandResult {
    let session_dir = session_directory();

    if arg.is_empty() {
        // List sessions
        let entries = match std::fs::read_dir(&session_dir) {
            Ok(entries) => entries,
            Err(_) => return CommandResult::Output(format!("  {DIM}No sessions found.{RESET}")),
        };

        let mut sessions: Vec<(String, std::time::SystemTime)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
                if let Some(time) = modified {
                    sessions.push((name, time));
                }
            }
        }

        sessions.sort_by(|a, b| b.1.cmp(&a.1));

        if sessions.is_empty() {
            return CommandResult::Output(format!("  {DIM}No sessions found.{RESET}"));
        }

        let mut output = format!("  {BOLD}Sessions{RESET}\n\n");
        for (i, (name, _)) in sessions.iter().take(10).enumerate() {
            output.push_str(&format!("  {DIM}{}.{RESET} {name}\n", i + 1));
        }
        if sessions.len() > 10 {
            output.push_str(&format!("  {DIM}... and {} more{RESET}\n", sessions.len() - 10));
        }

        CommandResult::Output(output)
    } else {
        // Load a specific session
        let path = session_dir.join(format!("{arg}.json"));
        if path.exists() {
            CommandResult::Output(format!("  {DIM}Session file: {}{RESET}", path.display()))
        } else {
            CommandResult::Output(format!("  {DIM}Session not found: {arg}{RESET}"))
        }
    }
}

fn session_directory() -> std::path::PathBuf {
    claude_rust_config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".claude-code-rs")
        .join("sessions")
}
