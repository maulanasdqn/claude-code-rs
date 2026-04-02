pub fn git_branch() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let branch = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if branch.is_empty() { None } else { Some(branch) }
            } else {
                None
            }
        })
}
