use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

/// Tool to delete an existing agent team.
///
/// TODO: Phase 5 — integrate with real coordinator/team manager.
pub struct TeamDeleteTool;

impl TeamDeleteTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for TeamDeleteTool {
    fn name(&self) -> &str {
        "team_delete"
    }

    fn description(&self) -> &str {
        "Delete an existing agent team by its ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "team_id": {
                    "type": "string",
                    "description": "The ID of the team to delete"
                }
            },
            "required": ["team_id"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let team_id = input
            .get("team_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'team_id' field".into()))?;

        // TODO: Phase 5 — replace stub with real coordinator integration
        tracing::info!(team_id, "deleting team (stub)");

        Ok(format!("Team '{team_id}' deleted successfully."))
    }
}
