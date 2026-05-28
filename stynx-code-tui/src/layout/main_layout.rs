use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutResult {
    pub sidebar: Option<Rect>,
    pub messages: Rect,
    pub thinking: Option<Rect>,
    pub delegate: Option<Rect>,
    pub input: Rect,
    pub footer: Rect,
}

pub struct MainLayout;

impl MainLayout {
    pub const SIDEBAR_WIDTH: u16 = 42;

    pub fn split(
        area: Rect,
        sidebar_visible: bool,
        input_lines: usize,
        thinking_lines: usize,
        delegate_lines: usize,
    ) -> LayoutResult {
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

        let content = input_lines.clamp(1, 8) as u16;
        let input_h = (content + 2).min(main.height.saturating_sub(4));
        let thinking_h: u16 = if thinking_lines == 0 {
            0
        } else {
            (1 + thinking_lines.min(4)) as u16
        };
        let delegate_h: u16 = delegate_lines.min(4) as u16;

        let mut constraints: Vec<Constraint> = vec![Constraint::Min(1)];
        if thinking_h > 0 { constraints.push(Constraint::Length(thinking_h)); }
        if delegate_h > 0 { constraints.push(Constraint::Length(delegate_h)); }
        constraints.push(Constraint::Length(input_h));
        constraints.push(Constraint::Length(1));

        let rows = Layout::vertical(constraints).split(main);
        let mut idx = 0;
        let messages = rows[idx]; idx += 1;
        let thinking = if thinking_h > 0 { let r = Some(rows[idx]); idx += 1; r } else { None };
        let delegate = if delegate_h > 0 { let r = Some(rows[idx]); idx += 1; r } else { None };
        let input = rows[idx]; idx += 1;
        let footer = rows[idx];

        LayoutResult { sidebar, messages, thinking, delegate, input, footer }
    }
}
