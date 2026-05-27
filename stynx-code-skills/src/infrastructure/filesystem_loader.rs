use std::path::{Path, PathBuf};

pub fn scan_skill_directories() -> Vec<PathBuf> {
    let home = stynx_code_config::home_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    vec![
        home.join(".claude").join("skills"),
        PathBuf::from(".claude/skills"),
        PathBuf::from(".claude/commands"),
    ]
}

pub fn find_skill_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.is_dir() {
        return results;
    }
    collect_skill_files(dir, &mut results);
    results
}

fn collect_skill_files(dir: &Path, results: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_skill_files(&path, results);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("md") {
                results.push(path);
            }
        }
    }
}
