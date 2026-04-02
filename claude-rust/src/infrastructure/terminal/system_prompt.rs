use std::path::Path;

use super::prompt_sections::{
    EnvInfo, actions_section, doing_tasks_section, efficiency_section,
    environment_section, intro_section, system_section, tone_section, using_tools_section,
};

fn today_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut days = secs / 86400;
    let mut year = 1970u32;
    loop {
        let in_year: u64 = if is_leap(year) { 366 } else { 365 };
        if days < in_year { break; }
        days -= in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let months: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for m in months {
        if days < m { break; }
        days -= m;
        month += 1;
    }
    format!("{year}-{month:02}-{:02}", days + 1)
}

fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Skill info for system prompt: (name, description, when_to_use)
pub fn make_system_prompt(env: &EnvInfo, tool_names: &[String], skills: &[(String, String, Option<String>)]) -> String {
    let today = today_date();

    let mut sections = vec![
        intro_section(),
        system_section(),
        doing_tasks_section(),
        actions_section(),
        using_tools_section(tool_names),
        tone_section(),
        efficiency_section(),
    ];

    if is_rust_project(&env.cwd) {
        sections.push(rust_section());
    }

    sections.push(format!(
        "# Session-specific guidance\n \
         - If you need the user to run a shell command themselves (e.g., an interactive login like `gcloud auth login`), suggest they type `! <command>` in the prompt — the `!` prefix runs the command in this session so its output lands directly in the conversation.\n \
         - Use the `agent` tool to delegate complex, independent subtasks to a sub-agent that has access to all tools. The sub-agent runs autonomously and returns its final response.\n \
         - Use the `explore` tool to spawn a read-only sub-agent specialized for codebase exploration (glob, grep, read). Use this when you need to research code without making changes.\n \
         - Sub-agents cannot spawn further sub-agents. Keep sub-agent tasks focused and self-contained."
    ));

    if !skills.is_empty() {
        let mut skill_section = "# User-defined Skills\nThe following custom skills are available as slash commands:\n".to_string();
        for (name, desc, when_to_use) in skills {
            skill_section.push_str(&format!(" - /{name}: {desc}\n"));
            if let Some(wtu) = when_to_use {
                skill_section.push_str(&format!("   When to use: {wtu}\n"));
            }
        }
        skill_section.push_str("\nWhen a task matches a skill's purpose, suggest the user invoke it with the slash command.");
        sections.push(skill_section);
    }

    sections.push("# Project-specific rules (always apply these)\n \
 - Never add code comments of any kind (no inline, no block, no doc comments) to any file\n \
 - Never add Claude branding or co-author lines to git commits\n \
 - No single source file may exceed 200 lines of code — split large files before they reach this limit\n \
 - Test files must be separate from feature/implementation files\n \
 - Always split code into small, single-responsibility files following Clean Architecture\n \
 - Design for Low Coupling and High Cohesion: each module should have one clear purpose with minimal dependencies on other modules".to_string());

    sections.push(environment_section(env));

    if let Some(ref gs) = env.git_status {
        sections.push(format!("gitStatus: {gs}"));
    }

    let claude_md = load_claude_md(&env.cwd);
    if !claude_md.is_empty() {
        sections.push(format!("# Project Instructions (CLAUDE.md)\n\n{claude_md}"));
    }

    sections.push(format!("# Current Date\nToday's date is {today}."));

    sections.join("\n\n")
}

fn rust_section() -> String {
    "# Rust Project Context

You are operating in a Rust project. Apply senior Rust engineering practices:
 - Prefer `cargo check` over `cargo build` for fast error feedback
 - Use `cargo clippy -- -D warnings` to enforce lint hygiene
 - Use `cargo test` to run tests; `cargo test -- --nocapture` to see println output
 - Use `cargo fmt` to format code
 - Understand and respect the borrow checker; don't fight it with unnecessary `clone()` or `Arc<Mutex<_>>`
 - Prefer `?` over `unwrap()`/`expect()` in library code; `expect()` is acceptable in binaries/tests
 - Use `cargo add <crate>` to add dependencies (requires cargo-edit)
 - For workspace projects, use `-p <crate>` to target specific crates
 - Understand Rust's ownership model: move semantics, lifetimes, borrowing
 - Prefer iterators and combinators over explicit loops when idiomatic
 - Use `#[derive(Debug, Clone, PartialEq)]` appropriately
 - For async code, prefer `tokio` patterns; avoid blocking in async contexts
 - When editing, always check that borrow/lifetime rules are satisfied before writing code
 - Run `cargo check` after edits to verify correctness before claiming done".to_string()
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
        for name in &["CLAUDE.md", "MEMORY.md"] {
            let candidate = d.join(name);
            if candidate.is_file() {
                if let Ok(text) = std::fs::read_to_string(&candidate) {
                    contents.push(text);
                }
            }
        }
        for name in &["CLAUDE.md", "MEMORY.md"] {
            let dotclaude = d.join(".claude").join(name);
            if dotclaude.is_file() {
                if let Ok(text) = std::fs::read_to_string(&dotclaude) {
                    contents.push(text);
                }
            }
        }
        if d == home.as_path() {
            break;
        }
        dir = d.parent();
    }

    let home_dotclaude_claude = home.join(".claude").join("CLAUDE.md");
    let home_dotclaude_memory = home.join(".claude").join("MEMORY.md");
    for path in [home_dotclaude_claude, home_dotclaude_memory] {
        if path.is_file() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if !contents.iter().any(|c| c == &text) {
                    contents.push(text);
                }
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

pub fn build_env_info(cwd: String, model_id: String) -> EnvInfo {
    use super::git::{git_status_snapshot, is_git_repo};

    let platform = std::env::consts::OS.to_string();
    let shell = std::env::var("SHELL")
        .map(|s| {
            if s.contains("zsh") { "zsh".to_string() }
            else if s.contains("bash") { "bash".to_string() }
            else { s }
        })
        .unwrap_or_else(|_| "unknown".to_string());

    let os_version = std::process::Command::new("uname")
        .args(["-sr"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| platform.clone());

    let is_git = is_git_repo(&cwd);
    let git_status = if is_git { git_status_snapshot(&cwd) } else { None };

    EnvInfo { cwd, is_git, platform, shell, os_version, model_id, git_status }
}
