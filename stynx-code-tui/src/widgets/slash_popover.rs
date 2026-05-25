use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::state::InputState;
use crate::theme;

pub struct SlashPopover<'a> {
    pub state: &'a InputState,
    pub anchor: Rect,
}

impl<'a> SlashPopover<'a> {
    pub fn new(state: &'a InputState, anchor: Rect) -> Self {
        Self { state, anchor }
    }
}

impl<'a> Widget for SlashPopover<'a> {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        if self.state.slash_matches.is_empty() { return; }
        let max_show = 8usize;
        let count = self.state.slash_matches.len().min(max_show) as u16;
        let max_cmd = self
            .state
            .slash_matches
            .iter()
            .map(|(c, _)| c.len())
            .max()
            .unwrap_or(8);
        let max_desc = self
            .state
            .slash_matches
            .iter()
            .map(|(_, d)| d.len())
            .max()
            .unwrap_or(20);
        let inner_w = (max_cmd + max_desc + 6) as u16;
        let width = inner_w.min(self.anchor.width.saturating_sub(2)).max(20);
        let height = count + 2;

        let x = self.anchor.x + 1;
        let y = self.anchor.y.saturating_sub(height);
        if y < 1 { return; }
        let rect = Rect { x, y, width, height };
        Clear.render(rect, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER_ACTIVE()))
            .style(Style::default().bg(theme::BACKGROUND_PANEL()));
        let inner = block.inner(rect);
        block.render(rect, buf);

        let mut lines: Vec<Line<'static>> = Vec::new();
        let selected = self.state.slash_selected.min(self.state.slash_matches.len().saturating_sub(1));
        for (i, (cmd, desc)) in self.state.slash_matches.iter().take(max_show).enumerate() {
            let is_sel = i == selected;
            let bg = if is_sel { theme::BACKGROUND_MENU() } else { theme::BACKGROUND_PANEL() };
            let cmd_padded = format!("{:width$}", cmd, width = max_cmd);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {cmd_padded}  "),
                    Style::default()
                        .fg(if is_sel { theme::ACCENT() } else { theme::TEXT() })
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    desc.clone(),
                    Style::default().fg(theme::TEXT_MUTED()).bg(bg),
                ),
            ]));
        }

        Paragraph::new(lines)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(inner, buf);
    }
}
