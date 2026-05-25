use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};

use crate::theme;

pub struct SearchOverlay<'a> {
    pub query: &'a str,
    pub results: &'a [String],
    pub selected: usize,
}

impl<'a> SearchOverlay<'a> {
    pub fn new(query: &'a str, results: &'a [String], selected: usize) -> Self {
        Self { query, results, selected }
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

impl<'a> Widget for SearchOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = centered_rect(60, 60, area);
        Clear.render(dialog, buf);

        let block = Block::default()
            .title(Span::styled(" Search History ", Style::default().fg(theme::GOLD()).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::GOLD()))
            .style(Style::default().bg(theme::SURFACE()));

        let inner = block.inner(dialog);
        block.render(dialog, buf);

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

        Paragraph::new(Line::from(vec![
            Span::styled("/ ", Style::default().fg(theme::GOLD())),
            Span::styled(self.query, Style::default().fg(theme::TEXT())),
        ])).block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme::HL_MED())))
            .render(chunks[0], buf);

        let items: Vec<ListItem<'_>> = self.results.iter().enumerate().map(|(i, result)| {
            let style = if i == self.selected {
                Style::default().fg(theme::BASE()).bg(theme::GOLD()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT())
            };
            ListItem::new(Line::from(Span::styled(result.as_str(), style)))
        }).collect();

        List::new(items).render(chunks[1], buf);
    }
}
