use std::path::{Path, PathBuf};

use crate::state::{AppState, DialogOption, SelectKind};

const SKIP_DIRS: &[&str] = &[
    ".git", "target", "node_modules", ".venv", "venv", "dist", "build",
    ".next", ".nuxt", ".turbo", ".cache", "__pycache__",
];

fn walk(root: &Path, base: &Path, out: &mut Vec<String>, depth: usize) {
    if depth > 6 || out.len() > 800 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else { return; };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') && name.as_ref() != ".env" { continue; }
        if SKIP_DIRS.contains(&name.as_ref()) { continue; }
        let path = entry.path();
        let Ok(meta) = entry.file_type() else { continue; };
        if meta.is_dir() {
            walk(&path, base, out, depth + 1);
        } else if meta.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.display().to_string());
            }
        }
        if out.len() > 800 { return; }
    }
}

pub fn open_file_mention(state: &mut AppState) {
    let cwd = PathBuf::from(if state.cwd.is_empty() { ".".into() } else { state.cwd.clone() });
    let mut files: Vec<String> = Vec::new();
    walk(&cwd, &cwd, &mut files, 0);
    files.sort();

    let options: Vec<DialogOption> = if files.is_empty() {
        vec![DialogOption::new("__empty__", "No files found")]
    } else {
        files
            .into_iter()
            .map(|p| DialogOption::new(p.clone(), p))
            .collect()
    };

    state.modal.open_select(
        SelectKind::FileMention,
        "Mention file",
        options,
        None,
        Some("↵ insert path".to_string()),
    );
}
