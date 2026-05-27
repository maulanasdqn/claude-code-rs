use crate::state::AppState;

pub fn open_status(state: &mut AppState, intern_labels: &[String]) {
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
    let intern_summary = if intern_labels.is_empty() {
        "unavailable".to_string()
    } else {
        format!("ready ({})", intern_labels.join(", "))
    };
    rows.push(("interns".into(), intern_summary));
    state.modal.open_info("Status", rows);
}
