use stynx_code_errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResponse {
    Yes,
    No,
    Always,
}

#[async_trait::async_trait]
pub trait ToolUI: Send + Sync {

    async fn ask_confirmation(&self, prompt: &str) -> AppResult<ConfirmResponse>;

    async fn ask_input(&self, prompt: &str, options: &[&str]) -> AppResult<String>;

    fn display_message(&self, message: &str);

    fn terminal_width(&self) -> usize;
}
