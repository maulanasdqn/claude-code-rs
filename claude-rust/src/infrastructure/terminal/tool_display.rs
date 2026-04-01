pub fn tool_icon(name: &str) -> &'static str {
    match name {
        "bash" => "⚡",
        "read" => "📄",
        "file_write" => "✏️",
        "file_edit" => "✏️",
        "glob" => "🔍",
        "grep" => "🔎",
        "ask_user_question" => "❓",
        "web_fetch" => "🌐",
        "web_search" => "🔍",
        "enter_plan_mode" => "📋",
        "exit_plan_mode" => "📋",
        _ => "⚙️",
    }
}

pub fn summarize_tool_input(name: &str, json: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.chars().take(80).collect(),
    };

    match name {
        "bash" => v
            .get("command")
            .and_then(|c| c.as_str())
            .map(|s| truncate_str(s, 80))
            .unwrap_or_default(),
        "read" => v
            .get("file_path")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        "file_write" => {
            let path = v.get("file_path").and_then(|p| p.as_str()).unwrap_or("?");
            format!("{path}")
        }
        "file_edit" => {
            let path = v.get("file_path").and_then(|p| p.as_str()).unwrap_or("?");
            format!("{path}")
        }
        "glob" => {
            let pat = v.get("pattern").and_then(|p| p.as_str()).unwrap_or("?");
            let base = v.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            format!("{pat} in {base}")
        }
        "grep" => {
            let pat = v.get("pattern").and_then(|p| p.as_str()).unwrap_or("?");
            let path = v.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            format!("/{pat}/ in {path}")
        }
        "ask_user_question" => v
            .get("question")
            .and_then(|q| q.as_str())
            .map(|s| truncate_str(s, 80))
            .unwrap_or_default(),
        "web_fetch" => v
            .get("url")
            .and_then(|u| u.as_str())
            .map(|s| truncate_str(s, 80))
            .unwrap_or_default(),
        "web_search" => v
            .get("query")
            .and_then(|q| q.as_str())
            .map(|s| truncate_str(s, 80))
            .unwrap_or_default(),
        "enter_plan_mode" | "exit_plan_mode" => String::new(),
        _ => truncate_str(json, 80),
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
