use serde_json::Value;

pub fn permission_title(tool_name: &str) -> String {
    match tool_name {
        "bash" => "Bash".into(),
        "file_write" => "Write".into(),
        "file_edit" => "Edit".into(),
        "web_search" => "Search".into(),
        "web_fetch" => "Fetch".into(),
        other => {
            let mut s = other.to_string();
            if let Some(c) = s.get_mut(0..1) { c.make_ascii_uppercase(); }
            s
        }
    }
}

pub fn format_permission_prompt(tool_name: &str, input: &Value) -> String {
    summarize_for_permission(tool_name, input)
}

fn summarize_for_permission(tool_name: &str, input: &Value) -> String {
    fn trunc(s: &str, n: usize) -> String {
        if s.len() > n { format!("{}…", &s[..n]) } else { s.to_string() }
    }
    match tool_name {
        "bash" => trunc(input.get("command").and_then(|v| v.as_str()).unwrap_or("(unknown)"), 100),
        "file_write" | "file_edit" => input.get("file_path").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
        "web_search" => trunc(input.get("query").and_then(|v| v.as_str()).unwrap_or("(unknown)"), 100),
        "web_fetch" => trunc(input.get("url").and_then(|v| v.as_str()).unwrap_or("(unknown)"), 100),
        _ => trunc(&input.to_string(), 100),
    }
}
