use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::state::InputState;
use crate::theme;

pub struct InputBox<'a> {
    pub state: &'a InputState,
    pub focused: bool,
}

impl<'a> InputBox<'a> {
    pub fn new(state: &'a InputState, focused: bool) -> Self {
        Self { state, focused }
    }
}

impl<'a> Widget for InputBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused { theme::IRIS() } else { theme::OVERLAY() };

        let hint = Span::styled(
            " ↵ send  esc · ",
            Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::DIM),
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                " › ",
                Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(hint);

        let inner = block.inner(area);

        let buffer_lines: Vec<&str> = if self.state.buffer.is_empty() {
            Vec::new()
        } else {
            self.state.buffer.split('\n').collect()
        };

        let lines: Vec<Line<'static>> = if buffer_lines.is_empty() {
            let hint = if self.state.suggestion.is_empty() {
                " Type a message…".to_string()
            } else {
                format!(" {}", self.state.suggestion)
            };
            vec![Line::from(Span::styled(
                hint,
                Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC),
            ))]
        } else {
            buffer_lines
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    if i == buffer_lines.len() - 1 && !self.state.suggestion.is_empty() {
                        Line::from(vec![
                            Span::styled(format!(" {l}"), Style::default().fg(theme::TEXT())),
                            Span::styled(
                                self.state.suggestion.clone(),
                                Style::default()
                                    .fg(theme::MUTED())
                                    .add_modifier(Modifier::DIM),
                            ),
                        ])
                    } else {
                        Line::from(Span::styled(format!(" {l}"), Style::default().fg(theme::TEXT())))
                    }
                })
                .collect()
        };

        Paragraph::new(lines)
            .block(block)
            .style(Style::default().bg(theme::BACKGROUND()))
            .render(area, buf);

        if self.focused && inner.width > 0 && inner.height > 0 {
            let (line, col) = self.state.cursor_line_col();
            let cx = inner.x.saturating_add(1).saturating_add(u16::try_from(col).unwrap_or(u16::MAX));
            let cy = inner.y.saturating_add(u16::try_from(line).unwrap_or(u16::MAX));
            let right = inner.x.saturating_add(inner.width);
            let bottom = inner.y.saturating_add(inner.height);
            if cx < right && cy < bottom {
                buf[(cx, cy)].set_style(Style::default().bg(theme::HL_HIGH()).fg(theme::TEXT()));
            }
        }
    }
}
