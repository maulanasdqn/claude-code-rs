use super::skill::{Skill, SkillMetadata, SkillSource};

pub fn bundled_skills() -> Vec<Skill> {
    vec![
        Skill {
            metadata: SkillMetadata {
                name: "commit".to_string(),
                description: "Create a git commit with a descriptive message based on the staged changes".to_string(),
                triggers: vec!["commit".to_string(), "git commit".to_string()],
                model: None,
                is_hidden: false,
            },
            content: "Create a git commit with a descriptive message based on the staged changes. {{ARGUMENTS}}".to_string(),
            source: SkillSource::Bundled,
        },
        Skill {
            metadata: SkillMetadata {
                name: "review".to_string(),
                description: "Review the code changes and provide constructive feedback".to_string(),
                triggers: vec!["review".to_string(), "code review".to_string()],
                model: None,
                is_hidden: false,
            },
            content: "Review the code changes and provide constructive feedback. {{ARGUMENTS}}".to_string(),
            source: SkillSource::Bundled,
        },
        Skill {
            metadata: SkillMetadata {
                name: "explain".to_string(),
                description: "Explain how this code works in detail".to_string(),
                triggers: vec!["explain".to_string(), "explain code".to_string()],
                model: None,
                is_hidden: false,
            },
            content: "Explain how this code works in detail. {{ARGUMENTS}}".to_string(),
            source: SkillSource::Bundled,
        },
    ]
}
