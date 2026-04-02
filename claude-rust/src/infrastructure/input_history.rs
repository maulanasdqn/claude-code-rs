use std::io::{BufRead, Write};
use std::path::PathBuf;

fn history_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join(".claude");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("history.jsonl"))
}

pub fn load_history(project: &str) -> Vec<String> {
    let Some(path) = history_path() else { return Vec::new() };
    let Ok(file) = std::fs::File::open(&path) else { return Vec::new() };
    let reader = std::io::BufReader::new(file);
    let mut entries: Vec<(u64, String)> = Vec::new();
    for line in reader.lines().map_while(|l| l.ok()) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
            let proj = val["project"].as_str().unwrap_or("");
            if proj != project {
                continue;
            }
            let display = val["display"].as_str().unwrap_or("").to_string();
            let ts = val["timestamp"].as_u64().unwrap_or(0);
            if !display.is_empty() {
                entries.push((ts, display));
            }
        }
    }
    entries.sort_by_key(|(ts, _)| *ts);
    let mut seen = std::collections::HashSet::new();
    entries.into_iter()
        .filter(|(_, d)| seen.insert(d.clone()))
        .map(|(_, d)| d)
        .collect()
}

pub fn append_history(entry: &str, project: &str) {
    let Some(path) = history_path() else { return };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let record = serde_json::json!({
        "display": entry,
        "project": project,
        "timestamp": ts,
    });
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{record}");
    }
}
