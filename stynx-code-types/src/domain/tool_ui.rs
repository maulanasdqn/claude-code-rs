use stynx_code_errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmResponse {
    Yes,
    No,
    Always,
}

impl std::fmt::Display for ConfirmResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => f.write_str("Yes"),
            Self::No => f.write_str("No"),
            Self::Always => f.write_str("Always"),
        }
    }
}

#[async_trait::async_trait]
pub trait ToolUI: Send + Sync {

    async fn ask_confirmation(&self, prompt: &str) -> AppResult<ConfirmResponse>;

    async fn ask_input(&self, prompt: &str, options: &[&str]) -> AppResult<String>;

    fn display_message(&self, message: &str);

    fn terminal_width(&self) -> usize;
}
