use std::path::PathBuf;

/// Returns the user's home directory.
/// Checks `HOME` first, then `USERPROFILE` (Windows).
pub fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

/// Returns `(program, flag)` for running a command string via the system shell.
///
/// Windows: `("cmd.exe", "/C")`
/// Unix: `("sh", "-c")`
pub fn shell_command() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd.exe", "/C")
    } else {
        ("sh", "-c")
    }
}
