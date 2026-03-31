use std::io::{self, BufRead, Write};
use std::path::Path;

// ── ANSI escape codes ──────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";

// ── Spinner verbs ──────────────────────────────────────────────────

pub const SPINNER_VERBS: &[&str] = &[
    "Thinking",
    "Reasoning",
    "Analyzing",
    "Processing",
    "Considering",
    "Evaluating",
    "Computing",
    "Reflecting",
    "Pondering",
    "Working",
];

// ── Terminal width ─────────────────────────────────────────────────

pub fn term_width() -> usize {
    // Try COLUMNS env, fall back to 80
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

// ── Banner ─────────────────────────────────────────────────────────

pub fn print_banner(cwd: &str) {
    let w = term_width().min(60);
    let rule = "─".repeat(w);

    println!();
    println!("  {DIM}{rule}{RESET}");
    println!("  {BOLD}{CYAN}◆  Claude Code{RESET} {DIM}(Rust) v0.2.0{RESET}");
    println!("  {DIM}{rule}{RESET}");
    println!();

    // Show cwd, truncated if too long
    let display_cwd = if cwd.len() > w - 6 {
        format!("...{}", &cwd[cwd.len() - (w - 9)..])
    } else {
        cwd.to_string()
    };
    println!("  {DIM}cwd:{RESET} {display_cwd}");
    println!("  {DIM}Type a message or{RESET} /help {DIM}for commands.{RESET}");
    println!();
}

// ── User input prompt ──────────────────────────────────────────────

pub fn read_user_input() -> Option<String> {
    print!("{BOLD}{CYAN}❯{RESET} ");
    io::stdout().flush().ok()?;

    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => {
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                Some(String::new())
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

// ── System prompt builder ──────────────────────────────────────────

pub fn make_system_prompt(cwd: &str) -> String {
    let mut prompt = format!(
        "You are Claude Code, an interactive CLI assistant for software engineering. \
         The user's working directory is: {cwd}. \
         You have access to the following tools:\n\
         - `bash`: Execute shell commands\n\
         - `read`: Read file contents with line numbers\n\
         - `file_write`: Write content to files (creates parent directories)\n\
         - `file_edit`: Edit files by exact string replacement (old_string must be unique)\n\
         - `glob`: Find files matching glob patterns (e.g. \"**/*.rs\")\n\
         - `grep`: Search file contents with regex (uses ripgrep)\n\
         - `ask_user_question`: Ask the user a question and wait for their response\n\
         - `web_fetch`: Fetch the content of a web page\n\
         - `web_search`: Search the web using DuckDuckGo\n\
         - `enter_plan_mode`: Enter plan mode (only read-only tools available)\n\
         - `exit_plan_mode`: Exit plan mode\n\n\
         Be concise and helpful. Use tools when needed to answer questions. \
         You can use multiple tools in a single response."
    );

    let claude_md_content = load_claude_md(cwd);
    if !claude_md_content.is_empty() {
        prompt.push_str("\n\n# Project Instructions\n\n");
        prompt.push_str(&claude_md_content);
    }

    if prompt.len() > 10_000 {
        prompt.truncate(10_000);
        prompt.push_str("\n... (truncated)");
    }

    prompt
}

fn load_claude_md(cwd: &str) -> String {
    let mut contents = Vec::new();
    let cwd_path = Path::new(cwd);
    let home = dirs_or_home();

    let mut dir = Some(cwd_path);
    while let Some(d) = dir {
        let candidate = d.join("CLAUDE.md");
        if candidate.is_file() {
            if let Ok(text) = std::fs::read_to_string(&candidate) {
                contents.push(text);
            }
        }
        let dotclaude = d.join(".claude").join("CLAUDE.md");
        if dotclaude.is_file() {
            if let Ok(text) = std::fs::read_to_string(&dotclaude) {
                contents.push(text);
            }
        }
        if d == home.as_path() {
            break;
        }
        dir = d.parent();
    }

    let home_dotclaude = home.join(".claude").join("CLAUDE.md");
    if home_dotclaude.is_file() {
        if let Ok(text) = std::fs::read_to_string(&home_dotclaude) {
            if !contents.iter().any(|c| c == &text) {
                contents.push(text);
            }
        }
    }

    contents.join("\n\n---\n\n")
}

fn dirs_or_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

// ── Session resume prompt ──────────────────────────────────────────

pub fn prompt_resume() -> bool {
    eprint!("  {DIM}Previous session found. Resume? {RESET}{BOLD}[y/n]{RESET} ");
    io::stderr().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        return line.trim().starts_with('y');
    }
    false
}

// ── Formatting helpers ─────────────────────────────────────────────

/// Format a tool name with its icon for display.
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

/// Abbreviate a tool's JSON input for one-line display.
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
