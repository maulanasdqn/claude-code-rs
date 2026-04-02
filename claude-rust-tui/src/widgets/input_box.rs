use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::state::{InputMode, InputState};

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
        let mode_label = match self.state.mode {
            InputMode::Insert => " INSERT ",
            InputMode::Normal => " NORMAL ",
            InputMode::Visual => " VISUAL ",
        };

        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(mode_label);

        let display = self.state.get_display_text();
        let text = if display.is_empty() {
            Line::from(Span::styled(
                "Type a message...",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Line::from(Span::raw(display))
        };

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);

        // Render cursor if focused
        if self.focused && area.width > 2 && area.height > 2 {
            let cursor_x = area.x + 1 + self.state.cursor_pos as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 {
                buf[(cursor_x, cursor_y)]
                    .set_style(Style::default().add_modifier(Modifier::REVERSED));
            }
        }
    }
}
