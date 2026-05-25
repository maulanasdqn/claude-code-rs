use crate::state::AppState;

pub fn open_status(state: &mut AppState, intern_label: Option<&str>) {
    let mut rows: Vec<(String, String)> = Vec::new();
    rows.push(("version".into(), env!("CARGO_PKG_VERSION").to_string()));
    rows.push(("cwd".into(), state.cwd.clone()));
    if let Some(b) = &state.git_branch {
        rows.push(("branch".into(), b.clone()));
    }
    rows.push(("model".into(), state.model_name.clone()));
    rows.push(("mode".into(), state.permission_mode.clone()));
    rows.push((
        "cost".into(),
        format!("${:.4}  ({} in / {} out)", state.total_cost, state.total_input, state.total_output),
    ));
    rows.push((
        "intern".into(),
        intern_label
            .map(|l| format!("ready ({l})"))
            .unwrap_or_else(|| "unavailable".into()),
    ));
    state.modal.open_info("Status", rows);
}
