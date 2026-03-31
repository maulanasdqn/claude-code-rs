use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct EnterPlanModeTool;

#[async_trait::async_trait]
impl Tool for EnterPlanModeTool {
    fn name(&self) -> &str {
        "enter_plan_mode"
    }

    fn description(&self) -> &str {
        "Enter plan mode. In plan mode, only read-only tools are available. Use this to safely explore and plan before making changes."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, _input: Value) -> AppResult<String> {
        Ok("Plan mode activated. Only read-only tools are now available. Use exit_plan_mode to return to normal mode.".into())
    }
}

pub struct ExitPlanModeTool;

#[async_trait::async_trait]
impl Tool for ExitPlanModeTool {
    fn name(&self) -> &str {
        "exit_plan_mode"
    }

    fn description(&self) -> &str {
        "Exit plan mode and return to normal mode where all tools are available."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, _input: Value) -> AppResult<String> {
        Ok("Exited plan mode. All tools are now available.".into())
    }
}
