use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Clone)]
#[allow(dead_code)]
pub enum InternStatus {
    Running,
    Completed,
    Failed(String),
    Killed,
    TimedOut,
}

pub struct InternRecord {
    pub id: String,
    pub intern: String,
    pub task_preview: String,
    pub status: InternStatus,
    pub result: Option<String>,
    pub started: Instant,
    pub last_action: Option<String>,
}

pub struct InternManager {
    records: Mutex<HashMap<String, InternRecord>>,
    channels: Mutex<HashMap<String, oneshot::Receiver<Result<String, String>>>>,
    handles: Mutex<HashMap<String, JoinHandle<()>>>,
    counter: Mutex<u64>,
}

impl InternManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            records: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            handles: Mutex::new(HashMap::new()),
            counter: Mutex::new(1),
        })
    }

    fn next_id(&self) -> String {
        let mut n = self.counter.lock().unwrap();
        let id = format!("intern-{n}");
        *n += 1;
        id
    }

    pub fn register(&self, intern: String, task: String) -> (String, oneshot::Sender<Result<String, String>>) {
        let id = self.next_id();
        let (tx, rx) = oneshot::channel();
        let preview = if task.chars().count() > 80 {
            format!("{}…", task.chars().take(79).collect::<String>())
        } else {
            task.clone()
        };
        self.records.lock().unwrap().insert(id.clone(), InternRecord {
            id: id.clone(),
            intern,
            task_preview: preview,
            status: InternStatus::Running,
            result: None,
            started: Instant::now(),
            last_action: None,
        });
        self.channels.lock().unwrap().insert(id.clone(), rx);
        (id, tx)
    }

    pub fn attach_handle(&self, id: &str, handle: JoinHandle<()>) {
        self.handles.lock().unwrap().insert(id.to_string(), handle);
    }

    #[allow(dead_code)]
    pub fn update_last_action(&self, id: &str, action: String) {
        if let Some(r) = self.records.lock().unwrap().get_mut(id) {
            r.last_action = Some(action);
        }
    }

    pub fn status_snapshot(&self, id: &str) -> Option<InternRecord> {
        self.records.lock().unwrap().get(id).map(|r| InternRecord {
            id: r.id.clone(),
            intern: r.intern.clone(),
            task_preview: r.task_preview.clone(),
            status: r.status.clone(),
            result: r.result.clone(),
            started: r.started,
            last_action: r.last_action.clone(),
        })
    }

    pub fn kill(&self, id: &str) -> bool {
        let handle = self.handles.lock().unwrap().remove(id);
        if let Some(h) = handle {
            h.abort();
            if let Some(r) = self.records.lock().unwrap().get_mut(id) {
                r.status = InternStatus::Killed;
            }
            self.channels.lock().unwrap().remove(id);
            true
        } else {
            false
        }
    }

    pub async fn wait_for(&self, id: &str, max_wait_secs: u64) -> Result<String, String> {
        let rx = self.channels.lock().unwrap().remove(id);
        let result = if let Some(rx) = rx {
            let timed = tokio::time::timeout(
                std::time::Duration::from_secs(max_wait_secs),
                rx,
            )
            .await;
            match timed {
                Ok(Ok(inner)) => inner,
                Ok(Err(_)) => Err("intern task was dropped".into()),
                Err(_) => {
                    return Err(format!("wait timed out after {max_wait_secs}s"));
                }
            }
        } else {
            self.records.lock().unwrap()
                .get(id)
                .and_then(|r| r.result.clone())
                .map(Ok)
                .unwrap_or_else(|| Err(format!("intern '{id}' not found or already consumed")))
        };
        {
            let mut records = self.records.lock().unwrap();
            if let Some(rec) = records.get_mut(id) {
                match &result {
                    Ok(out) => {
                        rec.status = InternStatus::Completed;
                        rec.result = Some(out.clone());
                    }
                    Err(e) => {
                        rec.status = InternStatus::Failed(e.clone());
                    }
                }
            }
        }
        self.handles.lock().unwrap().remove(id);
        result
    }

    pub fn snapshot(&self) -> Vec<InternSnapshot> {
        self.records.lock().unwrap().values().map(|r| InternSnapshot {
            id: r.id.clone(),
            intern: r.intern.clone(),
            task: r.task_preview.clone(),
            status: match &r.status {
                InternStatus::Running => "running".into(),
                InternStatus::Completed => "completed".into(),
                InternStatus::Failed(e) => format!("failed: {e}"),
                InternStatus::Killed => "killed".into(),
                InternStatus::TimedOut => "timed_out".into(),
            },
            elapsed_s: r.started.elapsed().as_secs(),
            last_action: r.last_action.clone(),
        }).collect()
    }
}

#[derive(serde::Serialize)]
pub struct InternSnapshot {
    pub id: String,
    pub intern: String,
    pub task: String,
    pub status: String,
    pub elapsed_s: u64,
    pub last_action: Option<String>,
}
