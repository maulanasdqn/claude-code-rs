use std::collections::VecDeque;
use std::sync::Mutex;

const MAX_EVENTS: usize = 1000;

#[derive(Debug, Clone)]
pub struct DiagnosticEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub details: String,
}

pub trait DiagnosticTracker: Send + Sync {
    fn record(&self, event: DiagnosticEvent);
    fn recent(&self, n: usize) -> Vec<DiagnosticEvent>;
}

pub struct InMemoryTracker {
    events: Mutex<VecDeque<DiagnosticEvent>>,
}

impl InMemoryTracker {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl DiagnosticTracker for InMemoryTracker {
    fn record(&self, mut event: DiagnosticEvent) {
        if event.timestamp == 0 {
            event.timestamp = now_secs();
        }
        let mut events = self.events.lock().unwrap_or_else(|e| e.into_inner());
        if events.len() >= MAX_EVENTS {
            events.pop_front();
        }
        events.push_back(event);
    }

    fn recent(&self, n: usize) -> Vec<DiagnosticEvent> {
        let events = self.events.lock().unwrap_or_else(|e| e.into_inner());
        events.iter().rev().take(n).cloned().collect()
    }
}
