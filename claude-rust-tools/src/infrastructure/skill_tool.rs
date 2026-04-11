use claude_rust_errors::AppResult;
use claude_rust_types::{PermissionLevel, SearchReadInfo, Tool};
use serde_json::{Value, json};

/// Tool to invoke a skill by reading its SKILL.md file.
pub struct SkillTool;

impl SkillTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        "Invoke a skill by name. Reads the skill definition from ~/.claude/skills/{name}/SKILL.md."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "description": "The skill name to invoke"
                },
                "args": {
                    "type": "string",
                    "description": "Optional arguments for the skill"
                }
            },
            "required": ["skill"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn is_concurrent_safe(&self, _input: &Value) -> bool { true }

    fn is_search_or_read_command(&self, _input: &Value) -> SearchReadInfo {
        SearchReadInfo { is_search: false, is_read: true, is_list: false }
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let skill = input
            .get("skill")
            .and_then(|v| v.as_str())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("missing 'skill' field".into()))?;

        let args = input
            .get("args")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(skill, args, "invoking skill");

        let home = claude_rust_config::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .ok_or_else(|| claude_rust_errors::AppError::Tool("cannot determine home directory".into()))?;

        let skill_path = format!("{home}/.claude/skills/{skill}/SKILL.md");

        let content = tokio::fs::read_to_string(&skill_path)
            .await
            .map_err(|e| claude_rust_errors::AppError::Tool(
                format!("skill '{skill}' not found at {skill_path}: {e}")
            ))?;

        if args.is_empty() {
            Ok(content)
        } else {
            Ok(format!("{content}\n\n---\nArgs: {args}"))
        }
    }
}
