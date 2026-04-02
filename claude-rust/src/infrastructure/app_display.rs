use claude_rust_types::{ContentBlock, Message, Role};

use super::terminal::{BOLD, CYAN, DIM, RESET};

pub fn show_transcript(conversation: &claude_rust_types::Conversation) {
    if conversation.messages.is_empty() {
        println!("\n  {DIM}No messages in conversation.{RESET}\n");
        return;
    }
    let mut output = String::new();
    for msg in &conversation.messages {
        let role_label = match msg.role {
            Role::User => format!("{BOLD}{CYAN}You{RESET}"),
            Role::Assistant => format!("{BOLD}Assistant{RESET}"),
        };
        output.push_str(&format!("\n{role_label}\n"));
        for block in &msg.content {
            match block {
                ContentBlock::Text { text } => { output.push_str(&format!("{text}\n")); }
                ContentBlock::ToolUse { name, .. } => { output.push_str(&format!("{DIM}[tool: {name}]{RESET}\n")); }
                ContentBlock::ToolResult { content, .. } => {
                    let preview = if content.len() > 100 { &content[..100] } else { content };
                    output.push_str(&format!("{DIM}[result: {preview}...]{RESET}\n"));
                }
                ContentBlock::Thinking { thinking } => {
                    let preview = if thinking.len() > 80 { &thinking[..80] } else { thinking };
                    output.push_str(&format!("{DIM}[thinking: {preview}...]{RESET}\n"));
                }
                ContentBlock::Image { .. } => { output.push_str(&format!("{DIM}[image]{RESET}\n")); }
            }
        }
        output.push_str(&format!("{DIM}────────────────────────────────{RESET}\n"));
    }
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less -R".into());
    let parts: Vec<&str> = pager.split_whitespace().collect();
    if let Some((bin, args)) = parts.split_first() {
        crossterm::terminal::disable_raw_mode().ok();
        if let Ok(mut child) = std::process::Command::new(bin).args(args)
            .stdin(std::process::Stdio::piped()).spawn()
        {
            if let Some(ref mut stdin) = child.stdin {
                use std::io::Write;
                let _ = stdin.write_all(output.as_bytes());
            }
            let _ = child.wait();
            crossterm::terminal::enable_raw_mode().ok();
            return;
        }
        crossterm::terminal::enable_raw_mode().ok();
    }
    print!("{output}");
}

pub fn show_todos(cwd: &str) {
    let todos_path = format!("{cwd}/.claude/todos.json");
    if !std::path::Path::new(&todos_path).exists() {
        println!("\n  {DIM}No todos found. Use the todo_write tool to create tasks.{RESET}\n");
        return;
    }
    match std::fs::read_to_string(&todos_path) {
        Ok(content) => {
            if let Ok(todos) = serde_json::from_str::<serde_json::Value>(&content) {
                println!("\n  {BOLD}{CYAN}Tasks{RESET}\n");
                if let Some(arr) = todos.as_array() {
                    if arr.is_empty() { println!("  {DIM}No tasks.{RESET}\n"); return; }
                    for (i, todo) in arr.iter().enumerate() {
                        let status = todo.get("status").and_then(|s| s.as_str()).unwrap_or("pending");
                        let content = todo.get("content").and_then(|s| s.as_str()).unwrap_or("(untitled)");
                        let icon = match status {
                            "completed" | "done" => "\x1b[32m✓\x1b[0m",
                            "in_progress" | "active" => "\x1b[33m●\x1b[0m",
                            _ => "\x1b[2m○\x1b[0m",
                        };
                        println!("  {icon} {}. {BOLD}{content}{RESET}", i + 1);
                    }
                    println!();
                } else if let Some(obj) = todos.as_object() {
                    if obj.is_empty() { println!("  {DIM}No tasks.{RESET}\n"); return; }
                    for (key, val) in obj {
                        let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("pending");
                        let content = val.get("content").and_then(|s| s.as_str()).unwrap_or(key.as_str());
                        let icon = match status {
                            "completed" | "done" => "\x1b[32m✓\x1b[0m",
                            "in_progress" | "active" => "\x1b[33m●\x1b[0m",
                            _ => "\x1b[2m○\x1b[0m",
                        };
                        println!("  {icon} {BOLD}{content}{RESET}");
                    }
                    println!();
                }
            } else { println!("\n  {DIM}Could not parse todos.json{RESET}\n"); }
        }
        Err(_) => println!("\n  {DIM}Could not read todos file.{RESET}\n"),
    }
}

pub fn print_session_history(messages: &[Message]) {
    let mut pairs: Vec<(&Message, Option<&Message>)> = Vec::new();
    let mut i = 0;
    while i < messages.len() {
        let msg = &messages[i];
        if matches!(msg.role, Role::User) {
            let next = messages.get(i + 1).filter(|m| matches!(m.role, Role::Assistant));
            pairs.push((msg, next));
            i += if next.is_some() { 2 } else { 1 };
        } else { i += 1; }
    }
    let start = pairs.len().saturating_sub(5);
    let pairs = &pairs[start..];
    if pairs.is_empty() { return; }
    let sep = format!("  {DIM}{}{RESET}", "─".repeat(60));
    for (user_msg, asst_msg) in pairs {
        println!("{sep}");
        let user_text: String = user_msg.content.iter().filter_map(|b| {
            if let ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
        }).collect::<Vec<_>>().join(" ");
        let first_line = user_text.lines().next().unwrap_or("").trim();
        let display = if first_line.len() > 80 { format!("{}…", &first_line[..79]) } else { first_line.to_string() };
        println!("  {BOLD}{CYAN}❯{RESET}  {DIM}{display}{RESET}");
        if let Some(asst) = asst_msg {
            let text: String = asst.content.iter().filter_map(|b| {
                if let ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
            }).collect::<Vec<_>>().join("\n");
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).take(3).collect();
            for line in &lines {
                let trimmed = line.trim();
                let d = if trimmed.len() > 100 { format!("{}…", &trimmed[..99]) } else { trimmed.to_string() };
                println!("  {DIM}{d}{RESET}");
            }
            if text.lines().filter(|l| !l.trim().is_empty()).count() > 3 {
                println!("  {DIM}…{RESET}");
            }
        }
    }
    println!("{sep}");
    println!();
}
