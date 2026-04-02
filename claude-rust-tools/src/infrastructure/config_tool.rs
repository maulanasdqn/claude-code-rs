use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

/// Tool for getting, setting, and listing configuration values.
///
/// TODO: Phase 4 — integrate with real configuration store.
pub struct ConfigTool;

impl ConfigTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for ConfigTool {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "Get, set, or list configuration values."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "Config operation: \"get\", \"set\", or \"list\"",
                    "enum": ["get", "set", "list"]
                },
                "key": {
                    "type": "string",
                    "description": "Configuration key (required for get/set)"
                },
                "value": {
                    "type": "string",
                    "description": "Configuration value (required for set)"
                }
            },
            "required": ["operation"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let operation = input
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'operation' field".into()))?;

        let key = input.get("key").and_then(|v| v.as_str());
        let value = input.get("value").and_then(|v| v.as_str());

        // TODO: Phase 4 — replace stub with real config store integration
        tracing::info!(operation, key, value, "config operation (stub)");

        match operation {
            "get" => {
                let key = key.ok_or_else(|| {
                    claude_rust_errors::AppError::Tool("'key' is required for get".into())
                })?;
                Ok(format!("Config '{key}': (not set — stub)"))
            }
            "set" => {
                let key = key.ok_or_else(|| {
                    claude_rust_errors::AppError::Tool("'key' is required for set".into())
                })?;
                let value = value.ok_or_else(|| {
                    claude_rust_errors::AppError::Tool("'value' is required for set".into())
                })?;
                Ok(format!("Config '{key}' set to '{value}'. (stub)"))
            }
            "list" => Ok("Configuration (stub):\n  (no entries)".to_string()),
            other => Err(claude_rust_errors::AppError::Tool(
                format!("unknown config operation: {other}")
            )),
        }
    }
}
