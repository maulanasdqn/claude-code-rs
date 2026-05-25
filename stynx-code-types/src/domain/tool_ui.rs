use stynx_code_errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResponse {
    Yes,
    No,
    Always,
}

#[async_trait::async_trait]
pub trait ToolUI: Send + Sync {
    /// Prompt user for yes/no/always confirmation
    async fn ask_confirmation(&self, prompt: &str) -> AppResult<ConfirmResponse>;
    /// Prompt user for free-text input
    async fn ask_input(&self, prompt: &str, options: &[&str]) -> AppResult<String>;
    /// Display a message to the user (non-blocking)
    fn display_message(&self, message: &str);
    /// Get terminal width (for formatting)
    fn terminal_width(&self) -> usize;
}
