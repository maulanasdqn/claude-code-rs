use async_trait::async_trait;
use stynx_code_errors::AppResult;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify(&self, title: &str, body: &str) -> AppResult<()>;
    async fn bell(&self) -> AppResult<()>;
}

pub struct TerminalNotifier;

impl TerminalNotifier {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NotificationService for TerminalNotifier {
    async fn notify(&self, title: &str, body: &str) -> AppResult<()> {

        if std::env::var("TERM_PROGRAM")
            .map(|v| v.contains("iTerm"))
            .unwrap_or(false)
        {
            eprint!("\x1b]9;{title}: {body}\x07");
        } else {
            eprintln!("[{title}] {body}");
        }
        tracing::info!(title, body, "sent notification");
        Ok(())
    }

    async fn bell(&self) -> AppResult<()> {
        eprint!("\x07");
        Ok(())
    }
}
