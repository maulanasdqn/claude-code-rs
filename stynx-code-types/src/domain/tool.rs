use stynx_code_errors::AppResult;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
    ReadOnly,
    Dangerous,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => f.write_str("ReadOnly"),
            Self::Dangerous => f.write_str("Dangerous"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptBehavior {
    Cancel,
    Block,
}

impl std::fmt::Display for InterruptBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancel => f.write_str("Cancel"),
            Self::Block => f.write_str("Block"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchReadInfo {
    pub is_search: bool,
    pub is_read: bool,
    pub is_list: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Ok,
    Error { message: String, error_code: i32 },
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => f.write_str("Ok"),
            Self::Error { message, error_code } => write!(f, "Error {}: {}", error_code, message),
        }
    }
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync {

    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn permission_level(&self) -> PermissionLevel;
    async fn execute(&self, input: Value) -> AppResult<String>;

    fn aliases(&self) -> &[&str] { &[] }
    fn search_hint(&self) -> Option<&str> { None }
    fn is_mcp(&self) -> bool { false }
    fn is_lsp(&self) -> bool { false }
    fn should_defer(&self) -> bool { false }
    fn always_load(&self) -> bool { false }

    fn is_read_only(&self, _input: &Value) -> bool { false }
    fn is_destructive(&self, _input: &Value) -> bool { false }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { false }
    fn is_enabled(&self) -> bool { true }
    fn interrupt_behavior(&self) -> InterruptBehavior { InterruptBehavior::Block }
    fn requires_user_interaction(&self) -> bool { false }
    fn is_open_world(&self, _input: &Value) -> bool { false }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: false, is_list: false }
    }

    fn max_result_size_chars(&self) -> usize { 100_000 }
    fn strict(&self) -> bool { false }

    fn backfill_observable_input(&self, _input: &mut Value) {}
    async fn validate_input(&self, _input: &Value) -> ValidationResult {
        ValidationResult::Ok
    }
    fn get_path(&self, _input: &Value) -> Option<String> { None }

    fn user_facing_name(&self, _input: &Value) -> String { self.name().to_string() }
    fn get_tool_use_summary(&self, _input: &Value) -> Option<String> { None }
    fn get_activity_description(&self, _input: &Value) -> Option<String> { None }
}
