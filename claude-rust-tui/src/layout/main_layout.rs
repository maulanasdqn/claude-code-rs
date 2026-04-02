use ratatui::layout::{Constraint, Layout, Rect};

pub struct MainLayout;

impl MainLayout {
    pub fn split(area: Rect) -> [Rect; 3] {
        let chunks = Layout::vertical([
            Constraint::Min(1),      // messages
            Constraint::Length(3),    // input
            Constraint::Length(1),    // status bar
        ])
        .split(area);

        [chunks[0], chunks[1], chunks[2]]
    }
}
