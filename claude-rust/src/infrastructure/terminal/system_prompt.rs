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
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
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

    sections.push("# Session-specific guidance\n \
         - For interactive shell commands users must run themselves, suggest `! <command>` in the prompt.\n \
         - Use `agent` to delegate independent subtasks. Use `explore` for read-only codebase research.\n \
         - Sub-agents cannot spawn further sub-agents.".to_string());

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

    sections.push("# Project-specific rules\n \
 - No code comments of any kind. No Claude branding in commits.\n \
 - Max 200 lines per source file — split before reaching the limit.\n \
 - Single-responsibility files. Low coupling, high cohesion.".to_string());

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
 - Use `cargo check` (not build) for fast feedback. `cargo clippy -- -D warnings` for lints. `cargo fmt` for formatting.
 - Respect the borrow checker. Prefer `?` over `unwrap`. Use `-p <crate>` for workspace targets.
 - Prefer iterators and combinators. Avoid blocking in async contexts.
 - Run `cargo check` after edits to confirm correctness.".to_string()
}

fn is_rust_project(cwd: &str) -> bool {
    let cwd_path = Path::new(cwd);
    if cwd_path.join("Cargo.toml").exists() {
        return true;
    }
    if let Some(parent) = cwd_path.parent()
        && parent.join("Cargo.toml").exists() {
            return true;
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
            if candidate.is_file()
                && let Ok(text) = std::fs::read_to_string(&candidate) {
                    contents.push(text);
                }
        }
        for name in &["CLAUDE.md", "MEMORY.md"] {
            let dotclaude = d.join(".claude").join(name);
            if dotclaude.is_file()
                && let Ok(text) = std::fs::read_to_string(&dotclaude) {
                    contents.push(text);
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
        if path.is_file()
            && let Ok(text) = std::fs::read_to_string(&path)
                && !contents.iter().any(|c| c == &text) {
                    contents.push(text);
                }
    }

    contents.join("\n\n---\n\n")
}

fn dirs_or_home() -> std::path::PathBuf {
    claude_rust_config::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

pub fn build_env_info(cwd: String, model_id: String) -> EnvInfo {
    use super::git::{git_status_snapshot, is_git_repo};

    let platform = std::env::consts::OS.to_string();

    let shell = if cfg!(windows) {
        if std::env::var("PSModulePath").is_ok() {
            "powershell".to_string()
        } else {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }
    } else {
        std::env::var("SHELL")
            .map(|s| {
                if s.contains("zsh") { "zsh".to_string() }
                else if s.contains("bash") { "bash".to_string() }
                else { s }
            })
            .unwrap_or_else(|_| "unknown".to_string())
    };

    let os_version = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "ver"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| platform.clone())
    } else {
        std::process::Command::new("uname")
            .args(["-sr"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| platform.clone())
    };

    let is_git = is_git_repo(&cwd);
    let git_status = if is_git { git_status_snapshot(&cwd) } else { None };

    EnvInfo { cwd, is_git, platform, shell, os_version, model_id, git_status }
}
