use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct LSPTool;

impl LSPTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for LSPTool {
    fn name(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Perform LSP operations such as diagnostics, hover, goto definition, references, and code actions."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "LSP operation: \"diagnostics\", \"hover\", \"goto_definition\", \"references\", or \"code_actions\"",
                    "enum": ["diagnostics", "hover", "goto_definition", "references", "code_actions"]
                },
                "file_path": {
                    "type": "string",
                    "description": "Path to the file for the LSP operation"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (0-indexed) for position-based operations"
                },
                "character": {
                    "type": "integer",
                    "description": "Character offset (0-indexed) for position-based operations"
                }
            },
            "required": ["operation", "file_path"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_lsp(&self) -> bool { true }
    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let operation = input
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'operation' field".into()))?;

        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'file_path' field".into()))?;

        let line = input.get("line").and_then(|v| v.as_u64());
        let character = input.get("character").and_then(|v| v.as_u64());

        tracing::info!(operation, file_path, ?line, ?character, "LSP operation (stub)");

        Ok(format!(
            "LSP server not running. Operation '{operation}' on '{file_path}' cannot be performed.\n\
             Hint: LSP integration will be available in Phase 4."
        ))
    }
}
