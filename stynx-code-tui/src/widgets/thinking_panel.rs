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
        if self.text.trim().is_empty() || area.height == 0 || area.width < 4 {
            return;
        }

        let frame = FRAMES[self.spinner_frame % FRAMES.len()];

        let visible_body = (area.height as usize).saturating_sub(1);
        let body_lines: Vec<&str> = {
            let mut v: Vec<&str> = self
                .text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();
            if v.len() > visible_body {
                v = v[v.len() - visible_body..].to_vec();
            }
            v
        };

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND()));
            }
        }

        let accent = theme::IRIS();
        let bar_col = theme::OVERLAY();

        for y in area.y..area.y + area.height {
            buf[(area.x, y)]
                .set_char('▌')
                .set_style(Style::default().fg(accent).bg(theme::BACKGROUND()));
        }

        let pad: u16 = if area.width >= 80 { 4 } else { 2 };
        let inner_x = area.x + pad;
        let inner_width = area.width.saturating_sub(pad as u16 + 1) as usize;
        let _ = bar_col;

        let header = Line::from(vec![
            Span::styled(
                format!("  {frame} "),
                Style::default()
                    .fg(accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "thinking",
                Style::default()
                    .fg(theme::SUBTLE())
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        let inner_area = Rect {
            x: inner_x,
            y: area.y,
            width: inner_width as u16,
            height: area.height,
        };

        let mut lines: Vec<Line<'static>> = vec![header];
        let body_inner_w = inner_width.saturating_sub(4);
        for raw in body_lines {
            let text = if raw.len() > body_inner_w {
                format!("{}…", &raw[..body_inner_w.saturating_sub(1)])
            } else {
                raw.trim_end().to_string()
            };
            lines.push(Line::from(Span::styled(
                format!("    {text}"),
                Style::default()
                    .fg(theme::MUTED())
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner_area, buf);
    }
}
