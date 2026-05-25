use stynx_code_errors::AppResult;

pub trait InputPreprocessor: Send + Sync {
    fn expand(&self, input: &str) -> AppResult<String>;
}
