pub fn tool_display_name(name: &str) -> String {
    match name {
        "bash" => "Bash",
        "read" => "Read",
        "file_write" => "Write",
        "file_edit" => "Edit",
        "glob" => "Glob",
        "grep" => "Grep",
        "web_fetch" => "Fetch",
        "web_search" => "Search",
        "ask_user_question" => "Ask",
        "enter_plan_mode" => "Plan",
        "exit_plan_mode" => "ExitPlan",
        "agent" => "Agent",
        "explore" => "Explore",
        "todo_write" => "TodoWrite",
        "todo_read" => "TodoRead",
        other => other,
    }
    .to_string()
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
            path.to_string()
        }
        "file_edit" => {
            let path = v.get("file_path").and_then(|p| p.as_str()).unwrap_or("?");
            path.to_string()
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
        "agent" | "explore" => v
            .get("task")
            .and_then(|t| t.as_str())
            .map(|s| truncate_str(s, 80))
            .unwrap_or_default(),
        "todo_read" => String::new(),
        n if n.starts_with("mcp__") => truncate_str(json, 80),
        "todo_write" => {
            let count = v
                .get("todos")
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            format!("{count} items")
        }
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
