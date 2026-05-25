use ratatui::layout::{Constraint, Layout, Rect};

pub struct ModalOverlay;

impl ModalOverlay {
    pub fn centered(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let vertical = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

        Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
    }
}
