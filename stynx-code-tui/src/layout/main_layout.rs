use ratatui::layout::{Constraint, Layout, Rect};

pub struct LayoutResult {
    /// Top of the left column — the tool stream.
    pub tool_history: Option<Rect>,
    /// Bottom of the left column — model / session info.
    pub tool_info: Option<Rect>,
    pub messages: Rect,
    pub thinking: Option<Rect>,
    pub delegate: Option<Rect>,
    pub summary: Option<Rect>,
    pub input: Rect,
    /// Fallback bottom bar — only used when the terminal is too narrow for the
    /// left sidebar (otherwise model info lives in `tool_info`).
    pub footer: Option<Rect>,
}

pub struct MainLayout;

impl MainLayout {
    pub const TOOL_HISTORY_WIDTH: u16 = 44;
    pub const MIN_MAIN_WIDTH: u16 = 60;
    /// Height reserved for the model-info section at the bottom of the sidebar.
    const INFO_HEIGHT: u16 = 9;

    pub fn split(
        area: Rect,
        input_lines: usize,
        thinking_lines: usize,
        delegate_lines: usize,
        has_summary: bool,
    ) -> LayoutResult {
        let (tool_col, main) = if area.width > Self::TOOL_HISTORY_WIDTH + Self::MIN_MAIN_WIDTH {
            let chunks = Layout::horizontal([
                Constraint::Length(Self::TOOL_HISTORY_WIDTH),
                Constraint::Min(1),
            ])
            .split(area);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, area)
        };

        // Split the left column: tool stream on top, model info on the bottom.
        let (tool_history, tool_info) = match tool_col {
            Some(col) => {
                let info_h = Self::INFO_HEIGHT.min(col.height.saturating_sub(3));
                if info_h >= 4 {
                    let parts = Layout::vertical([
                        Constraint::Min(1),
                        Constraint::Length(info_h),
                    ])
                    .split(col);
                    (Some(parts[0]), Some(parts[1]))
                } else {
                    (Some(col), None)
                }
            }
            None => (None, None),
        };

        let content = input_lines.clamp(1, 8) as u16;
        let input_h = (content + 2).min(main.height.saturating_sub(4));
        let thinking_h: u16 = if thinking_lines == 0 {
            0
        } else {
            (1 + thinking_lines.min(12)) as u16
        };
        let delegate_h: u16 = delegate_lines.min(4) as u16;
        // Show a bottom bar only when there's no sidebar to host the model info.
        let footer_h: u16 = if tool_info.is_some() { 0 } else { 1 };

        let mut constraints: Vec<Constraint> = vec![Constraint::Min(1)];
        if thinking_h > 0 { constraints.push(Constraint::Length(thinking_h)); }
        if delegate_h > 0 { constraints.push(Constraint::Length(delegate_h)); }
        if has_summary { constraints.push(Constraint::Length(1)); }
        constraints.push(Constraint::Length(input_h));
        if footer_h > 0 { constraints.push(Constraint::Length(footer_h)); }

        let rows = Layout::vertical(constraints).split(main);
        let mut idx = 0;
        let messages = rows[idx]; idx += 1;
        let thinking = if thinking_h > 0 { let r = Some(rows[idx]); idx += 1; r } else { None };
        let delegate = if delegate_h > 0 { let r = Some(rows[idx]); idx += 1; r } else { None };
        let summary = if has_summary { let r = Some(rows[idx]); idx += 1; r } else { None };
        let input = rows[idx]; idx += 1;
        let footer = if footer_h > 0 { Some(rows[idx]) } else { None };

        LayoutResult { tool_history, tool_info, messages, thinking, delegate, summary, input, footer }
    }
}
