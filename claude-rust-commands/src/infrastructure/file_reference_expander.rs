use claude_rust_errors::AppResult;
use crate::application::expand_file_references;
use crate::domain::InputPreprocessor;

pub struct FileReferenceExpander;

impl InputPreprocessor for FileReferenceExpander {
    fn expand(&self, input: &str) -> AppResult<String> {
        Ok(expand_file_references(input))
    }
}
