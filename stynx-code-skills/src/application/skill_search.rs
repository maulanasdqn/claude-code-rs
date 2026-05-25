use crate::domain::skill::Skill;

/// Case-insensitive search across skill name, description, and triggers.
pub fn search_skills<'a>(skills: &'a [Skill], query: &str) -> Vec<&'a Skill> {
    let q = query.to_lowercase();
    skills
        .iter()
        .filter(|skill| {
            skill.metadata.name.to_lowercase().contains(&q)
                || skill.metadata.description.to_lowercase().contains(&q)
                || skill
                    .metadata
                    .triggers
                    .iter()
                    .any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}
