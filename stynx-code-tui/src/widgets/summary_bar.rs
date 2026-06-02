use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;

pub struct SummaryBar<'a> {
    items: &'a [String],
}

impl<'a> SummaryBar<'a> {
    pub fn new(items: &'a [String]) -> Self {
        Self { items }
    }
}

impl<'a> Widget for SummaryBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = theme::SURFACE();

        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(bg));
        }

        let check = Span::styled(
            " ✓ ",
            Style::default()
                .fg(theme::SUCCESS())
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        );

        let mut spans: Vec<Span<'static>> = vec![check];

        let pill_bg = theme::OVERLAY();
        let pill_fg = theme::TEXT();
        let sep_fg = theme::MUTED();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    "  ",
                    Style::default().fg(sep_fg).bg(bg),
                ));
            }
            spans.push(Span::styled(
                format!(" {} ", item),
                Style::default()
                    .fg(pill_fg)
                    .bg(pill_bg)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(bg))
            .render(area, buf);
    }
}
