use std::sync::Arc;

use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionChecker, PermissionDecision, PermissionLevel, ValidationResult};
use claude_rust_tools::ToolRegistry;

use crate::application::undo::{UndoStack, push_file_state};

pub async fn execute_tool(
    registry: &ToolRegistry,
    permission: &Arc<dyn PermissionChecker>,
    name: &str,
    input: &serde_json::Value,
    undo_stack: &Arc<UndoStack>,
) -> AppResult<String> {
    let tool = registry
        .get(name)
        .ok_or_else(|| AppError::Tool(format!("unknown tool: {name}")))?;

    // Validate input before execution
    if let ValidationResult::Error { message, .. } = tool.validate_input(input).await {
        return Err(AppError::Tool(format!("validation error for {name}: {message}")));
    }

    if tool.permission_level() == PermissionLevel::Dangerous {
        let decision = permission.check(name, input).await?;
        if let PermissionDecision::Deny(reason) = decision {
            return Err(AppError::PermissionDenied(reason));
        }
    }

    // Use get_path() for undo instead of hardcoded tool name matching
    if (tool.is_destructive(input) || matches!(name, "file_edit" | "file_write"))
        && let Some(path) = tool.get_path(input)
    {
        push_file_state(undo_stack, &path).await;
    }

    tool.execute(input.clone()).await
}

pub fn is_overloaded(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("overloaded") || lower.contains("529") || lower.contains("rate")
}
