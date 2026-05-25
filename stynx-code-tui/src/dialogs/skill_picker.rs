use crate::state::{AppState, DialogOption, SelectKind};

pub fn open_skill_picker(state: &mut AppState, skills: &[(String, String)]) {
    let options: Vec<DialogOption> = if skills.is_empty() {
        vec![DialogOption::new("__empty__", "No skills available")
            .with_description("Define skills under .claude/skills to populate this list.")]
    } else {
        skills
            .iter()
            .map(|(name, desc)| {
                DialogOption::new(format!("skill:{name}"), format!("/{name}"))
                    .with_description(desc.clone())
                    .with_category("Skills")
            })
            .collect()
    };
    state.modal.open_select(
        SelectKind::SkillPicker,
        "Skills",
        options,
        None,
        Some("↵ insert into input".to_string()),
    );
}
