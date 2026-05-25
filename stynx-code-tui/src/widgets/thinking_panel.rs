use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct ThinkingPanel<'a> {
    pub text: &'a str,
    pub spinner_frame: usize,
}

impl<'a> ThinkingPanel<'a> {
    pub fn new(text: &'a str, spinner_frame: usize) -> Self {
        Self { text, spinner_frame }
    }
}

impl<'a> Widget for ThinkingPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.text.trim().is_empty() || area.height == 0 {
            return;
        }

        // Subtle backdrop so the panel reads as a distinct surface.
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND_PANEL()));
            }
        }

        let visible_body = (area.height as usize).saturating_sub(1);
        let mut body_lines: Vec<&str> = self
            .text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if body_lines.len() > visible_body {
            body_lines = body_lines[body_lines.len() - visible_body..].to_vec();
        }

        let frame = FRAMES[self.spinner_frame % FRAMES.len()];

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {frame}  "),
                Style::default()
                    .fg(theme::PRIMARY())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "thinking",
                Style::default()
                    .fg(theme::TEXT_MUTED())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
        for raw in body_lines {
            lines.push(Line::from(vec![
                Span::styled(
                    "     ",
                    Style::default().bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    raw.trim_end().to_string(),
                    Style::default()
                        .fg(theme::TEXT_MUTED())
                        .bg(theme::BACKGROUND_PANEL())
                        .add_modifier(Modifier::ITALIC | Modifier::DIM),
                ),
            ]));
        }

        Paragraph::new(lines)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}
