use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;

const FRAMES: [char; 10] = [
    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}',
    '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}',
    '\u{2807}', '\u{280F}',
];

pub struct Spinner {
    pub frame: usize,
}

impl Spinner {
    pub fn new(frame: usize) -> Self { Self { frame } }
    pub fn current_char(&self) -> char { FRAMES[self.frame % FRAMES.len()] }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line = Line::from(Span::styled(
            format!(" {}", self.current_char()),
            Style::default().fg(theme::IRIS).add_modifier(Modifier::BOLD),
        ));
        Paragraph::new(line).render(area, buf);
    }
}
