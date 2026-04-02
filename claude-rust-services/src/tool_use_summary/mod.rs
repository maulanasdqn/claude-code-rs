pub trait ToolUseSummarizer: Send + Sync {
    fn summarize(&self, tool_name: &str, input: &str, output: &str) -> String;
}

pub struct DefaultSummarizer;

impl DefaultSummarizer {
    pub fn new() -> Self {
        Self
    }
}

impl ToolUseSummarizer for DefaultSummarizer {
    fn summarize(&self, tool_name: &str, _input: &str, output: &str) -> String {
        let truncated = if output.len() > 200 {
            format!("{}...", &output[..200])
        } else {
            output.to_string()
        };
        format!("{tool_name}: {truncated}")
    }
}
