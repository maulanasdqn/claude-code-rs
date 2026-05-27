use stynx_code_errors::AppResult;
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct TeamCreateTool;

impl TeamCreateTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for TeamCreateTool {
    fn name(&self) -> &str {
        "team_create"
    }

    fn description(&self) -> &str {
        "Create a new agent team with a name and description. Returns a generated team ID."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name for the new team"
                },
                "description": {
                    "type": "string",
                    "description": "Description of the team's purpose"
                }
            },
            "required": ["name", "description"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let name = input
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'name' field".into()))?;

        let description = input
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| stynx_code_errors::AppError::Tool("missing 'description' field".into()))?;

        let team_id = format!("team-{:08x}", rand_id());

        tracing::info!(name, description, team_id, "creating team (stub)");

        Ok(format!("Team created.\n  id: {team_id}\n  name: {name}\n  description: {description}"))
    }
}

fn rand_id() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    h.finish() as u32
}
