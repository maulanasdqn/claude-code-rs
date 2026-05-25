use crate::state::{AppState, DialogOption, SelectKind};

const KEYS: &[(&str, &str)] = &[
    ("Enter", "Submit message"),
    ("Esc", "Vim normal mode / close modal"),
    ("Shift+Tab", "Cycle permission mode"),
    ("Ctrl+C", "Quit"),
    ("Ctrl+P", "Command palette"),
    ("Ctrl+S", "Session list"),
    ("Ctrl+M", "Model picker"),
    ("Ctrl+B", "Toggle sidebar"),
    ("PgUp / PgDn", "Scroll messages by page"),
    ("Shift+↑ / Shift+↓", "Scroll messages by line"),
    ("Ctrl+Alt+u / Ctrl+Alt+d", "Scroll messages by half page"),
    ("Tab", "Accept input suggestion"),
    ("Ctrl+U", "Clear input"),
    ("Ctrl+W", "Delete previous word"),
    ("Ctrl+A / Home", "Cursor to line start"),
    ("Ctrl+E / End", "Cursor to line end"),
    ("Alt+← / Alt+→", "Word jump"),
];

const SLASH: &[(&str, &str)] = &[
    ("/help", "Show this help"),
    ("/intern <task>", "Delegate task to intern model (DeepSeek)"),
    ("/quit, /exit", "Exit session"),
    ("/version", "Show version"),
    ("/model [name]", "Switch model"),
    ("/mode", "Cycle permission mode"),
    ("/think", "Toggle extended thinking"),
    ("/effort low|medium|high|max", "Set effort level"),
    ("/fast", "Toggle fast mode"),
    ("/plan [task]", "Toggle plan mode"),
    ("/compact", "Compact context"),
    ("/cost", "Show usage and cost"),
    ("/diff", "Show git diff"),
    ("/status", "Show git status"),
    ("/review", "Review current diff"),
    ("/commit", "Generate commit message"),
    ("/memory", "Show CLAUDE.md"),
    ("/init", "Create / update CLAUDE.md"),
    ("/add <path>", "Pin a file to every message"),
    ("/files", "List pinned files"),
    ("/skills", "List available skills"),
    ("/session", "Manage sessions"),
    ("/rewind [n]", "Remove last n exchanges"),
    ("/export", "Export conversation"),
    ("/copy", "Copy last response"),
    ("/undo [n]", "Restore last n file edits"),
    ("/config", "Show merged config"),
    ("/permissions", "Show allow / deny rules"),
];

pub fn open_help(state: &mut AppState, skills: &[(String, String)]) {
    let mut options: Vec<DialogOption> = Vec::new();
    for (key, desc) in KEYS {
        options.push(
            DialogOption::new(format!("k:{key}"), key.to_string())
                .with_description(desc.to_string())
                .with_category("Keyboard"),
        );
    }
    for (cmd, desc) in SLASH {
        options.push(
            DialogOption::new(format!("s:{cmd}"), cmd.to_string())
                .with_description(desc.to_string())
                .with_category("Slash"),
        );
    }
    for (name, desc) in skills {
        options.push(
            DialogOption::new(format!("sk:{name}"), format!("/{name}"))
                .with_description(desc.clone())
                .with_category("Skills"),
        );
    }
    state.modal.open_select(
        SelectKind::Help,
        "Help",
        options,
        None,
        Some("type to filter".to_string()),
    );
}
