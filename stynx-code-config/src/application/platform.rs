use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

pub fn shell_command() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd.exe", "/C")
    } else {
        ("sh", "-c")
    }
}
