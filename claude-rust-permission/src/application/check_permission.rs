use serde_json::Value;

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";

pub fn format_permission_prompt(tool_name: &str, input: &Value) -> String {
    let summary = summarize_for_permission(tool_name, input);
    format!(
        "\n  {YELLOW}{BOLD}⚠  {summary}{RESET}\n  {DIM}Allow?{RESET} {BOLD}[y/n]{RESET} "
    )
}

fn summarize_for_permission(tool_name: &str, input: &Value) -> String {
    match tool_name {
        "bash" => {
            let cmd = input
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("(unknown)");
            let display = if cmd.len() > 100 {
                format!("{}…", &cmd[..99])
            } else {
                cmd.to_string()
            };
            format!("Run: {DIM}{display}{RESET}")
        }
        "file_write" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("?");
            format!("Write: {DIM}{path}{RESET}")
        }
        "file_edit" => {
            let path = input
                .get("file_path")
                .and_then(|p| p.as_str())
                .unwrap_or("?");
            format!("Edit: {DIM}{path}{RESET}")
        }
        "web_search" => {
            let query = input
                .get("query")
                .and_then(|q| q.as_str())
                .unwrap_or("(unknown)");
            let display = if query.len() > 100 {
                format!("{}…", &query[..99])
            } else {
                query.to_string()
            };
            format!("Search: {DIM}{display}{RESET}")
        }
        "web_fetch" => {
            let url = input
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("(unknown)");
            let display = if url.len() > 100 {
                format!("{}…", &url[..99])
            } else {
                url.to_string()
            };
            format!("Fetch: {DIM}{display}{RESET}")
        }
        _ => {
            let s = input.to_string();
            let preview = if s.len() > 100 {
                format!("{}…", &s[..99])
            } else {
                s
            };
            format!("{tool_name}: {DIM}{preview}{RESET}")
        }
    }
}
