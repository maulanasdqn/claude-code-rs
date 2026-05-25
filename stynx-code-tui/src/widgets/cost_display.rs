use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;

pub struct CostDisplay {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
}

impl CostDisplay {
    pub fn new(input_tokens: u64, output_tokens: u64, cost: f64) -> Self {
        Self { input_tokens, output_tokens, cost }
    }

    fn format_tokens(count: u64) -> String {
        if count >= 1_000_000 { format!("{:.1}M", count as f64 / 1_000_000.0) }
        else if count >= 1_000 { format!("{:.1}K", count as f64 / 1_000.0) }
        else { count.to_string() }
    }
}

impl Widget for CostDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(vec![
            Span::styled("In: ", Style::default().fg(theme::MUTED())),
            Span::styled(Self::format_tokens(self.input_tokens), Style::default().fg(theme::FOAM())),
            Span::styled("  Out: ", Style::default().fg(theme::MUTED())),
            Span::styled(Self::format_tokens(self.output_tokens), Style::default().fg(theme::FOAM())),
            Span::styled("  Cost: ", Style::default().fg(theme::MUTED())),
            Span::styled(format!("${:.4}", self.cost), Style::default().fg(theme::GOLD())),
        ]);
        Paragraph::new(line).render(area, buf);
    }
}
