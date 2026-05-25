use std::sync::Arc;
use tokio::sync::Mutex;

pub struct UndoEntry {
    pub path: String,
    pub original: Option<String>,
}

pub type UndoStack = Mutex<Vec<UndoEntry>>;

pub async fn push_file_state(stack: &Arc<UndoStack>, path: &str) {
    let original = tokio::fs::read_to_string(path).await.ok();
    stack.lock().await.push(UndoEntry { path: path.to_string(), original });
}

pub async fn restore(stack: &Arc<UndoStack>, n: usize) -> Vec<(String, bool)> {
    let mut guard = stack.lock().await;
    let take = n.min(guard.len());
    if take == 0 { return Vec::new(); }
    let len = guard.len();
    let entries: Vec<_> = guard.drain(len - take..).collect();
    drop(guard);
    let mut results = Vec::new();
    for entry in entries.into_iter().rev() {
        let ok = match entry.original {
            Some(c) => tokio::fs::write(&entry.path, c).await.is_ok(),
            None => tokio::fs::remove_file(&entry.path).await.is_ok(),
        };
        results.push((entry.path, ok));
    }
    results
}
