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
            InputMode::Insert => (" INSERT ", theme::PINE),
            InputMode::Normal => (" NORMAL ", theme::GOLD),
        };

        let border_color = if self.focused { theme::PINE } else { theme::HL_MED };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(mode_label, Style::default().fg(mode_color).add_modifier(Modifier::BOLD)));

        let display = self.state.get_display_text();
        let text = if display.is_empty() && self.state.suggestion.is_empty() {
            Line::from(Span::styled("Type a message...", Style::default().fg(theme::MUTED)))
        } else if self.state.suggestion.is_empty() {
            Line::from(Span::styled(display, Style::default().fg(theme::TEXT)))
        } else {
            Line::from(vec![
                Span::styled(display, Style::default().fg(theme::TEXT)),
                Span::styled(self.state.suggestion.clone(), Style::default().fg(theme::MUTED).add_modifier(Modifier::DIM)),
            ])
        };

        Paragraph::new(text)
            .block(block)
            .style(Style::default().bg(theme::SURFACE))
            .render(area, buf);

        if self.focused && area.width > 2 && area.height > 2 {
            let cursor_x = area.x + 1 + self.state.cursor_pos as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 {
                buf[(cursor_x, cursor_y)].set_style(Style::default().bg(theme::HL_HIGH).fg(theme::TEXT));
            }
        }
    }
}
