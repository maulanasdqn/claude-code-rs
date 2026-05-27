use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillId(pub String);

impl SkillId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SkillId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub is_hidden: bool,
}

#[derive(Debug, Clone)]
pub enum SkillSource {
    UserSkill(PathBuf),
    ProjectSkill(PathBuf),
    Bundled,
    Plugin(String),
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub metadata: SkillMetadata,

    pub content: String,
    pub source: SkillSource,
}

impl Skill {

    pub fn expand_template(&self, arguments: &str) -> String {
        self.content.replace("{{ARGUMENTS}}", arguments)
    }
}
