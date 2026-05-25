use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::theme;

pub struct InfoDialog<'a> {
    pub title: &'a str,
    pub rows: &'a [(String, String)],
}

impl<'a> InfoDialog<'a> {
    pub fn new(title: &'a str, rows: &'a [(String, String)]) -> Self {
        Self { title, rows }
    }
}

impl<'a> Widget for InfoDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_key = self.rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        let visible_rows = self.rows.len() as u16 + 4;
        let h = visible_rows.min(area.height.saturating_sub(4));
        let w = 60.min(area.width.saturating_sub(6));
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

        let mut lines: Vec<Line<'static>> = Vec::new();
        for (k, v) in self.rows {
            let key_padded = format!("{:width$}", k, width = max_key);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {key_padded}  "),
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    v.clone(),
                    Style::default()
                        .fg(theme::TEXT())
                        .bg(theme::BACKGROUND_PANEL())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " esc  close ".to_string(),
            Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
        )));

        Paragraph::new(lines)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(inner, buf);
    }
}
