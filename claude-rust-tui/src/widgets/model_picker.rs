use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

use crate::theme;

pub struct ModelPicker<'a> {
    pub options: &'a [String],
    pub selected: usize,
}

impl<'a> ModelPicker<'a> {
    pub fn new(options: &'a [String], selected: usize) -> Self {
        Self { options, selected }
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

impl<'a> Widget for ModelPicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = centered_rect(40, 50, area);
        Clear.render(dialog, buf);

        let block = Block::default()
            .title(Span::styled(" Select Model ", Style::default().fg(theme::IRIS).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::IRIS))
            .style(Style::default().bg(theme::SURFACE));

        let items: Vec<ListItem<'_>> = self.options.iter().enumerate().map(|(i, opt)| {
            let (prefix, style) = if i == self.selected {
                ("› ", Style::default().fg(theme::BASE).bg(theme::IRIS).add_modifier(Modifier::BOLD))
            } else {
                ("  ", Style::default().fg(theme::TEXT))
            };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{opt}"), style)))
        }).collect();

        List::new(items).block(block).render(dialog, buf);
    }
}
