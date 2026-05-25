use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::{AppState, DialogOption, SelectKind};

fn human_age(seconds_ago: u64) -> String {
    match seconds_ago {
        0..=59 => "just now".into(),
        60..=3599 => format!("{}m ago", seconds_ago / 60),
        3600..=86_399 => format!("{}h ago", seconds_ago / 3600),
        _ => format!("{}d ago", seconds_ago / 86_400),
    }
}

pub fn open_session_list(state: &mut AppState) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut entries: Vec<DialogOption> = Vec::new();

    let mut pinned: Vec<_> = state
        .sidebar
        .sessions
        .iter()
        .filter(|s| s.pinned)
        .collect();
    pinned.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    for s in pinned {
        let opt = DialogOption::new(&s.id, &s.title)
            .with_category("Pinned")
            .with_footer(human_age(now.saturating_sub(s.updated_at)));
        entries.push(opt);
    }

    let mut today: Vec<_> = state
        .sidebar
        .sessions
        .iter()
        .filter(|s| !s.pinned)
        .collect();
    today.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    for s in today {
        let opt = DialogOption::new(&s.id, &s.title)
            .with_category("Recent")
            .with_footer(human_age(now.saturating_sub(s.updated_at)));
        entries.push(opt);
    }

    if entries.is_empty() {
        entries.push(
            DialogOption::new("__empty__", "No sessions yet")
                .with_description("Start a conversation to populate this list."),
        );
    }

    let current = if state.sidebar.session_id.is_empty() {
        None
    } else {
        Some(state.sidebar.session_id.clone())
    };

    state.modal.open_select(
        SelectKind::SessionList,
        "Sessions",
        entries,
        current,
        Some("^D delete  ^R rename".to_string()),
    );
}
