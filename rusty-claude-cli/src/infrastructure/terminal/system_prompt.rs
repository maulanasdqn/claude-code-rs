use std::path::Path;

pub fn build_system_prompt(cwd: &str, model_id: &str) -> String {
    let today = today_date();

    let mut sections = vec![
        "You are Claude, an AI assistant by Anthropic. \
         You help with software engineering tasks. Be concise and direct."
            .to_string(),
        format!("Working directory: {cwd}"),
        format!("Model: {model_id}"),
    ];

    if is_rust_project(cwd) {
        sections.push(
            "# Rust Project\n\
             Use `cargo check` for fast feedback. `cargo clippy -- -D warnings` for lints.\n\
             Prefer `?` over `unwrap`. Use iterators and combinators."
                .to_string(),
        );
    }

    let claude_md = load_claude_md(cwd);
    if !claude_md.is_empty() {
        sections.push(format!("# Project Instructions (CLAUDE.md)\n\n{claude_md}"));
    }

    sections.push(format!("# Current Date\nToday's date is {today}."));

    sections.join("\n\n")
}

fn today_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut days = secs / 86400;
    let mut year = 1970u32;
    loop {
        let in_year: u64 = if is_leap(year) { 366 } else { 365 };
        if days < in_year {
            break;
        }
        days -= in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let months: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for m in months {
        if days < m {
            break;
        }
        days -= m;
        month += 1;
    }
    format!("{year}-{month:02}-{:02}", days + 1)
}

fn is_leap(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn is_rust_project(cwd: &str) -> bool {
    let cwd_path = Path::new(cwd);
    if cwd_path.join("Cargo.toml").exists() {
        return true;
    }
    if let Some(parent) = cwd_path.parent()
        && parent.join("Cargo.toml").exists()
    {
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
                && let Ok(text) = std::fs::read_to_string(&candidate)
            {
                contents.push(text);
            }
        }
        if d == home.as_path() {
            break;
        }
        dir = d.parent();
    }
    contents.join("\n\n---\n\n")
}

fn dirs_or_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}
