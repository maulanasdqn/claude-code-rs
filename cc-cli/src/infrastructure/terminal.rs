use std::io::{self, BufRead, Write};

pub const BLUE: &str = "\x1b[34m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";

pub fn print_banner() {
    println!("{BOLD}{BLUE}╭─────────────────────────────────╮{RESET}");
    println!("{BOLD}{BLUE}│  Claude Code (Rust)  v0.1.0     │{RESET}");
    println!("{BOLD}{BLUE}╰─────────────────────────────────╯{RESET}");
    println!("{DIM}Type a message to chat. /help for commands. Ctrl+C to exit.{RESET}");
    println!();
}

pub fn read_user_input() -> Option<String> {
    print!("{BOLD}{GREEN}> {RESET}");
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

pub fn make_system_prompt(cwd: &str) -> String {
    format!(
        "You are Claude Code, an interactive CLI assistant. \
         The user's working directory is: {cwd}. \
         You have access to `bash` and `read` tools. \
         Be concise and helpful. Use tools when needed to answer questions."
    )
}

pub fn prompt_resume() -> bool {
    eprint!("{DIM}Previous session found. Resume? [y/n]: {RESET}");
    io::stderr().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        return line.trim().starts_with('y');
    }
    false
}
