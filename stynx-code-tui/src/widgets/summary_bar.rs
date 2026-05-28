use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use crate::theme;

pub struct SummaryBar<'a> {
    pub text: &'a str,
}

impl<'a> SummaryBar<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Widget for SummaryBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pad = if area.width >= 80 { "    " } else { "  " };
        let line = Line::from(vec![
            Span::styled(format!("{pad}✓ "), Style::default().fg(theme::SUCCESS()).add_modifier(Modifier::BOLD)),
            Span::styled(self.text.to_string(), Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::ITALIC)),
        ]);
        Paragraph::new(line)
            .block(Block::default().style(Style::default().bg(theme::SURFACE())))
            .render(area, buf);
    }
}
