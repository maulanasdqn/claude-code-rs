use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub expires_at: Instant,
}

#[derive(Default)]
pub struct ToastState {
    pub items: Vec<Toast>,
}

impl ToastState {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, message: impl Into<String>, kind: ToastKind, ttl: Duration) {
        let toast = Toast {
            message: message.into(),
            kind,
            expires_at: Instant::now() + ttl,
        };
        self.items.push(toast);
        if self.items.len() > 4 {
            self.items.remove(0);
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Info, Duration::from_secs(3));
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Success, Duration::from_secs(3));
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Warning, Duration::from_secs(4));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message, ToastKind::Error, Duration::from_secs(5));
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.items.retain(|t| t.expires_at > now);
    }
}
