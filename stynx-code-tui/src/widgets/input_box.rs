use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::state::{InputMode, InputState};
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
        let (mode_label, mode_color) = match self.state.mode {
            InputMode::Insert => (" INSERT ", theme::PINE()),
            InputMode::Normal => (" NORMAL ", theme::GOLD()),
        };

        let border_color = if self.focused { theme::PINE() } else { theme::HL_MED() };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                mode_label,
                Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);

        let buffer_lines: Vec<&str> = if self.state.buffer.is_empty() {
            Vec::new()
        } else {
            self.state.buffer.split('\n').collect()
        };

        let lines: Vec<Line<'static>> = if buffer_lines.is_empty() {
            let hint = if self.state.suggestion.is_empty() {
                "Type a message…".to_string()
            } else {
                self.state.suggestion.clone()
            };
            vec![Line::from(Span::styled(
                hint,
                Style::default().fg(theme::MUTED()),
            ))]
        } else {
            buffer_lines
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    if i == buffer_lines.len() - 1 && !self.state.suggestion.is_empty() {
                        Line::from(vec![
                            Span::styled(l.to_string(), Style::default().fg(theme::TEXT())),
                            Span::styled(
                                self.state.suggestion.clone(),
                                Style::default()
                                    .fg(theme::MUTED())
                                    .add_modifier(Modifier::DIM),
                            ),
                        ])
                    } else {
                        Line::from(Span::styled(l.to_string(), Style::default().fg(theme::TEXT())))
                    }
                })
                .collect()
        };

        Paragraph::new(lines)
            .block(block)
            .style(Style::default().bg(theme::SURFACE()))
            .render(area, buf);

        if self.focused && inner.width > 0 && inner.height > 0 {
            let (line, col) = self.state.cursor_line_col();
            let cx = inner.x + col as u16;
            let cy = inner.y + line as u16;
            if cx < inner.x + inner.width && cy < inner.y + inner.height {
                buf[(cx, cy)].set_style(Style::default().bg(theme::HL_HIGH()).fg(theme::TEXT()));
            }
        }
    }
}
