use crate::state::{AppState, DialogOption, SelectKind};

pub struct PaletteCommand {
    pub name: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub category: &'static str,
    pub shortcut: Option<&'static str>,
}

pub const COMMANDS: &[PaletteCommand] = &[
    PaletteCommand {
        name: "session.new",
        title: "New session",
        desc: "Start a new conversation",
        category: "Session",
        shortcut: Some("^X n"),
    },
    PaletteCommand {
        name: "session.list",
        title: "Switch session",
        desc: "Open the session list",
        category: "Session",
        shortcut: Some("^S"),
    },
    PaletteCommand {
        name: "session.compact",
        title: "Compact session",
        desc: "Summarize older turns to free up context",
        category: "Session",
        shortcut: Some("^X c"),
    },
    PaletteCommand {
        name: "session.export",
        title: "Export session",
        desc: "Write the transcript to disk",
        category: "Session",
        shortcut: None,
    },
    PaletteCommand {
        name: "session.rename",
        title: "Rename session",
        desc: "Set a new title for this session",
        category: "Session",
        shortcut: None,
    },
    PaletteCommand {
        name: "status.show",
        title: "Show status",
        desc: "Display current env, model, mode, cost",
        category: "Session",
        shortcut: None,
    },
    PaletteCommand {
        name: "skills.show",
        title: "Skills",
        desc: "Browse available skills",
        category: "Session",
        shortcut: None,
    },
    PaletteCommand {
        name: "model.list",
        title: "Switch model",
        desc: "Pick a different model for this session",
        category: "Model",
        shortcut: Some("^M"),
    },
    PaletteCommand {
        name: "model.cycle_recent",
        title: "Cycle recent model",
        desc: "Quick-switch to the previous model",
        category: "Model",
        shortcut: Some("F2"),
    },
    PaletteCommand {
        name: "mode.cycle",
        title: "Cycle permission mode",
        desc: "Normal → Auto-accept → Plan → Bypass",
        category: "Mode",
        shortcut: Some("S-Tab"),
    },
    PaletteCommand {
        name: "sidebar.toggle",
        title: "Toggle sidebar",
        desc: "Show or hide the left panel",
        category: "View",
        shortcut: Some("^B"),
    },
    PaletteCommand {
        name: "theme.switch",
        title: "Switch theme",
        desc: "Pick a different theme",
        category: "View",
        shortcut: None,
    },
    PaletteCommand {
        name: "help.show",
        title: "Help",
        desc: "Show keyboard shortcuts and slash commands",
        category: "Help",
        shortcut: Some("?"),
    },
    PaletteCommand {
        name: "app.quit",
        title: "Quit",
        desc: "Exit stynx-code",
        category: "App",
        shortcut: Some("^C"),
    },
];

pub fn open_command_palette(state: &mut AppState) {
    let options: Vec<DialogOption> = COMMANDS
        .iter()
        .map(|c| {
            let mut opt = DialogOption::new(c.name, c.title)
                .with_description(c.desc)
                .with_category(c.category);
            if let Some(s) = c.shortcut {
                opt = opt.with_footer(s);
            }
            opt
        })
        .collect();
    state
        .modal
        .open_select(SelectKind::CommandPalette, "Commands", options, None, None);
}
