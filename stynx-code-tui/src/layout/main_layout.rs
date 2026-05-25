use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutResult {
    pub sidebar: Option<Rect>,
    pub messages: Rect,
    pub input: Rect,
    pub footer: Rect,
}

pub struct MainLayout;

impl MainLayout {
    pub const SIDEBAR_WIDTH: u16 = 42;

    pub fn split(area: Rect, sidebar_visible: bool, input_lines: usize) -> LayoutResult {
        let (sidebar, main) = if sidebar_visible && area.width > Self::SIDEBAR_WIDTH + 20 {
            let chunks = Layout::horizontal([
                Constraint::Length(Self::SIDEBAR_WIDTH),
                Constraint::Min(1),
            ])
            .split(area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, area)
        };

        // Input box border = 2 rows; content rows = input_lines clamped to 1..=8
        let content = input_lines.clamp(1, 8) as u16;
        let input_h = (content + 2).min(main.height.saturating_sub(4));

        let rows = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(input_h),
            Constraint::Length(1),
        ])
        .split(main);

        LayoutResult {
            sidebar,
            messages: rows[0],
            input: rows[1],
            footer: rows[2],
        }
    }
}
