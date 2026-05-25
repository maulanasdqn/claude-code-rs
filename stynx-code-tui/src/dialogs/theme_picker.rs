use crate::state::{AppState, DialogOption, SelectKind};
use crate::theme;

pub fn open_theme_picker(state: &mut AppState) {
    let current = theme::current().id.to_string();
    let options: Vec<DialogOption> = theme::THEMES
        .iter()
        .map(|t| {
            DialogOption::new(t.id, t.name)
                .with_category("Theme")
        })
        .collect();
    state.modal.open_select(
        SelectKind::ThemePicker,
        "Themes",
        options,
        Some(current),
        Some("↵ apply".to_string()),
    );
}
