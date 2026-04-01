use std::path::Path;

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

    if is_rust_project(cwd) {
        prompt.push_str(
            "\n\n# Rust Project Context\n\n\
             You are operating in a Rust project. Apply senior Rust engineering practices:\n\
             - Prefer `cargo check` over `cargo build` for fast error feedback\n\
             - Use `cargo clippy -- -D warnings` to enforce lint hygiene\n\
             - Use `cargo test` to run tests; `cargo test -- --nocapture` to see println output\n\
             - Use `cargo fmt` to format code\n\
             - Understand and respect the borrow checker; don't fight it with unnecessary `clone()` or `Arc<Mutex<_>>`\n\
             - Prefer `?` over `unwrap()`/`expect()` in library code; `expect()` is acceptable in binaries/tests\n\
             - Use `cargo add <crate>` to add dependencies (requires cargo-edit)\n\
             - For workspace projects, use `-p <crate>` to target specific crates\n\
             - Understand Rust's ownership model: move semantics, lifetimes, borrowing\n\
             - Prefer iterators and combinators over explicit loops when idiomatic\n\
             - Use `#[derive(Debug, Clone, PartialEq)]` appropriately\n\
             - For async code, prefer `tokio` patterns; avoid blocking in async contexts\n\
             - When editing, always check that borrow/lifetime rules are satisfied before writing code\n\
             - Run `cargo check` after edits to verify correctness before claiming done"
        );
    }

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

fn is_rust_project(cwd: &str) -> bool {
    let cwd_path = Path::new(cwd);
    if cwd_path.join("Cargo.toml").exists() {
        return true;
    }
    if let Some(parent) = cwd_path.parent() {
        if parent.join("Cargo.toml").exists() {
            return true;
        }
    }
    false
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
