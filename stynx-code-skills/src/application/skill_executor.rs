use stynx_code_errors::AppResult;

use crate::domain::skill::Skill;

pub trait SkillExecutor {
    fn execute(&self, skill: &Skill, args: &str) -> AppResult<String>;
}

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
