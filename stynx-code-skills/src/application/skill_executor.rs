use stynx_code_errors::AppResult;

use crate::domain::skill::Skill;

/// Trait for executing a skill with optional arguments.
pub trait SkillExecutor {
    fn execute(&self, skill: &Skill, args: &str) -> AppResult<String>;
}

/// Simple executor that returns the expanded template content.
///
/// Real execution with engine integration is handled at a higher layer.
pub struct SimpleSkillExecutor;

impl SimpleSkillExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSkillExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillExecutor for SimpleSkillExecutor {
    fn execute(&self, skill: &Skill, args: &str) -> AppResult<String> {
        Ok(skill.expand_template(args))
    }
}
