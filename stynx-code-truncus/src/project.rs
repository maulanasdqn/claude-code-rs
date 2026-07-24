/// Derive a project name from a working directory — the final path component,
/// matching how the Truncus worker keys sessions/lessons/knowledge by project.
pub fn project_from_cwd(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
