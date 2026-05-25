use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::theme;

pub struct InputDialog<'a> {
    pub title: &'a str,
    pub prompt: &'a str,
    pub buffer: &'a str,
}

impl<'a> InputDialog<'a> {
    pub fn new(title: &'a str, prompt: &'a str, buffer: &'a str) -> Self {
        Self { title, prompt, buffer }
    }
}

impl<'a> Widget for InputDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let w = 60.min(area.width.saturating_sub(4));
        let h = 7.min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let rect = Rect { x, y, width: w, height: h };
        Clear.render(rect, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER_ACTIVE()))
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default().fg(theme::TEXT()).add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(rect);
        block.render(rect, buf);

        let lines: Vec<Line<'static>> = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", self.prompt),
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  › ",
                    Style::default().fg(theme::ACCENT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    self.buffer.to_string(),
                    Style::default()
                        .fg(theme::TEXT())
                        .bg(theme::BACKGROUND_PANEL())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "▏",
                    Style::default().fg(theme::ACCENT()).bg(theme::BACKGROUND_PANEL()),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  enter  confirm   esc  cancel ".to_string(),
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
            )),
        ];

        Paragraph::new(lines)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(inner, buf);
    }
}
