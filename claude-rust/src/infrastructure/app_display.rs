use claude_rust_types::{ContentBlock, Message, Role};

use super::terminal::{BOLD, CYAN, DIM, RESET};

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
