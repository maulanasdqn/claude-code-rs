use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::theme;

pub struct PermissionDialog<'a> {
    pub tool_name: &'a str,
    pub description: &'a str,
}

impl<'a> PermissionDialog<'a> {
    pub fn new(tool_name: &'a str, description: &'a str) -> Self {
        Self { tool_name, description }
    }
}

fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ]).split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ]).split(v[1])[1]
}

impl<'a> Widget for PermissionDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = centered_rect(50, 40, area);
        Clear.render(dialog, buf);

        let block = Block::default()
            .title(Span::styled(" Permission Required ", Style::default().fg(theme::LOVE).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::LOVE))
            .style(Style::default().bg(theme::SURFACE));

        let inner = block.inner(dialog);
        block.render(dialog, buf);

        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ]).split(inner);

        Paragraph::new(Line::from(vec![
            Span::styled("Tool: ", Style::default().fg(theme::MUTED)),
            Span::styled(self.tool_name, Style::default().fg(theme::FOAM).add_modifier(Modifier::BOLD)),
        ])).render(chunks[0], buf);

        Paragraph::new(self.description)
            .style(Style::default().fg(theme::TEXT))
            .wrap(Wrap { trim: true })
            .render(chunks[1], buf);

        Paragraph::new(Line::from(vec![
            Span::styled("[Y]", Style::default().fg(theme::FOAM).add_modifier(Modifier::BOLD)),
            Span::styled("es  ", Style::default().fg(theme::SUBTLE)),
            Span::styled("[N]", Style::default().fg(theme::LOVE).add_modifier(Modifier::BOLD)),
            Span::styled("o  ", Style::default().fg(theme::SUBTLE)),
            Span::styled("[A]", Style::default().fg(theme::GOLD).add_modifier(Modifier::BOLD)),
            Span::styled("lways", Style::default().fg(theme::SUBTLE)),
        ])).alignment(Alignment::Center).render(chunks[2], buf);
    }
}
