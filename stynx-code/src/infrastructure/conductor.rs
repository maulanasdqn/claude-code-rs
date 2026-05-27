use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;
use std::time::Instant;

use stynx_code_config::HooksConfig;
use stynx_code_types::PermissionChecker;
use stynx_code_tools::ToolRegistry;

pub enum AgentStatus {
    Running,
    Completed,
    Failed(String),
}

pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub task_preview: String,
    pub status: AgentStatus,
    pub result: Option<String>,
    pub started: Instant,
}

pub struct AgentManager {
    records: Mutex<HashMap<String, AgentRecord>>,
    channels: Mutex<HashMap<String, tokio::sync::oneshot::Receiver<Result<String, String>>>>,
    counter: Mutex<u64>,
}

impl AgentManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            records: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            counter: Mutex::new(1),
        })
    }

    fn next_id(&self) -> String {
        let mut n = self.counter.lock().unwrap();
        let id = format!("agent-{n}");
        *n += 1;
        id
    }

    pub fn register(&self, name: String, task: String) -> (String, tokio::sync::oneshot::Sender<Result<String, String>>) {
        let id = self.next_id();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let preview = if task.len() > 80 { format!("{}…", &task[..79]) } else { task.clone() };
        self.records.lock().unwrap().insert(id.clone(), AgentRecord {
            id: id.clone(), name, task_preview: preview,
            status: AgentStatus::Running, result: None, started: Instant::now(),
        });
        self.channels.lock().unwrap().insert(id.clone(), rx);
        (id, tx)
    }

    pub async fn wait_for(&self, id: &str) -> Result<String, String> {
        let rx = self.channels.lock().unwrap().remove(id);
        let result = if let Some(rx) = rx {
            rx.await.unwrap_or_else(|_| Err("agent task was dropped".into()))
        } else {
            self.records.lock().unwrap()
                .get(id).and_then(|r| r.result.clone()).map(Ok)
                .unwrap_or_else(|| Err(format!("agent '{id}' not found or already consumed")))
        };
        {
            let mut records = self.records.lock().unwrap();
            if let Some(rec) = records.get_mut(id) {
                match &result {
                    Ok(out) => { rec.status = AgentStatus::Completed; rec.result = Some(out.clone()); }
                    Err(e) => { rec.status = AgentStatus::Failed(e.clone()); }
                }
            }
        }
        result
    }

    pub fn snapshot(&self) -> Vec<AgentSnapshot> {
        self.records.lock().unwrap().values().map(|r| AgentSnapshot {
            id: r.id.clone(), name: r.name.clone(), task: r.task_preview.clone(),
            status: match &r.status {
                AgentStatus::Running => "running".into(),
                AgentStatus::Completed => "completed".into(),
                AgentStatus::Failed(e) => format!("failed: {e}"),
            },
            elapsed_s: r.started.elapsed().as_secs(),
        }).collect()
    }
}

#[derive(serde::Serialize)]
pub struct AgentSnapshot {
    pub id: String,
    pub name: String,
    pub task: String,
    pub status: String,
    pub elapsed_s: u64,
}

#[derive(Clone)]
pub(crate) struct AgentCtx {
    pub provider: Arc<dyn stynx_code_types::Provider>,
    pub registry: Arc<ToolRegistry>,
    pub permission: Arc<dyn PermissionChecker>,
    pub mode: Arc<AtomicU8>,
    pub hooks: HooksConfig,
}
