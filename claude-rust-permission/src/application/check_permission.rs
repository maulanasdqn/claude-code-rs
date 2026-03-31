use serde_json::Value;

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

pub fn format_permission_prompt(tool_name: &str, input: &Value) -> String {
    let summary = summarize_for_permission(tool_name, input);

    format!(
        "\n  {YELLOW}{BOLD}⚠  Permission required{RESET}\n\
         {DIM}  ├─{RESET} Tool: {CYAN}{BOLD}{tool_name}{RESET}\n\
         {DIM}  ├─{RESET} {summary}\n\
         {DIM}  └─{RESET} Allow? {BOLD}[y/n]{RESET} "
    )
}

fn summarize_for_permission(tool_name: &str, input: &Value) -> String {
    match tool_name {
        "bash" => {
            let cmd = input
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("(unknown)");
            let display = if cmd.len() > 120 {
                format!("{}…", &cmd[..119])
            } else {
                cmd.to_string()
            };
            format!("Command: {DIM}{display}{RESET}")
        }
        "file_write" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("?");
            format!("Write to: {DIM}{path}{RESET}")
        }
        "file_edit" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("?");
            format!("Edit: {DIM}{path}{RESET}")
        }
        _ => {
            let s = input.to_string();
            let preview = if s.len() > 120 {
                format!("{}…", &s[..119])
            } else {
                s
            };
            format!("Input: {DIM}{preview}{RESET}")
        }
    }
}
