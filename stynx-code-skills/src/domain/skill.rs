use std::path::PathBuf;
use serde::Deserialize;

/// Newtype wrapper for a skill identifier.
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

/// Metadata parsed from the YAML frontmatter of a skill file.
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

/// Where a skill was loaded from.
#[derive(Debug, Clone)]
pub enum SkillSource {
    UserSkill(PathBuf),
    ProjectSkill(PathBuf),
    Bundled,
    Plugin(String),
}

/// A fully loaded skill with metadata, markdown body, and its origin.
#[derive(Debug, Clone)]
pub struct Skill {
    pub metadata: SkillMetadata,
    /// The full markdown body (everything after the frontmatter).
    pub content: String,
    pub source: SkillSource,
}

impl Skill {
    /// Replace `{{ARGUMENTS}}` in the content with the given arguments string.
    pub fn expand_template(&self, arguments: &str) -> String {
        self.content.replace("{{ARGUMENTS}}", arguments)
    }
}
