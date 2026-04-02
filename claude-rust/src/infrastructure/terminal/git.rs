pub fn git_branch() -> Option<String> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let head = dir.join(".git").join("HEAD");
        if head.exists() {
            let content = std::fs::read_to_string(head).ok()?;
            let content = content.trim();
            if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                return Some(branch.to_string());
            }
            if content.len() >= 7 {
                return Some(format!("({})", &content[..7]));
            }
            return None;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

pub fn is_git_repo(cwd: &str) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn git_status_snapshot(cwd: &str) -> Option<String> {
    let run = |args: &[&str]| -> String {
        std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    };

    let branch = run(&["rev-parse", "--abbrev-ref", "HEAD"]);
    if branch.is_empty() {
        return None;
    }

    let main_branch = ["main", "master", "develop"]
        .iter()
        .find(|&&b| {
            std::process::Command::new("git")
                .args(["show-ref", "--verify", &format!("refs/heads/{b}")])
                .current_dir(cwd)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| branch.clone());

    let status = {
        let raw = run(&["--no-optional-locks", "status", "--short"]);
        if raw.len() > 2000 {
            format!("{}\n... (truncated)", &raw[..2000])
        } else if raw.is_empty() {
            "(clean)".to_string()
        } else {
            raw
        }
    };

    let log = run(&["--no-optional-locks", "log", "--oneline", "-n", "5"]);
    let user = run(&["config", "user.name"]);

    let mut parts = vec![
        "This is the git status at the start of the conversation. Note that this status is a snapshot in time, and will not update during the conversation.".to_string(),
        format!("Current branch: {branch}"),
        format!("Main branch (you will usually use this for PRs): {main_branch}"),
    ];
    if !user.is_empty() {
        parts.push(format!("Git user: {user}"));
    }
    parts.push(format!("Status:\n{status}"));
    if !log.is_empty() {
        parts.push(format!("Recent commits:\n{log}"));
    }

    Some(parts.join("\n\n"))
}
