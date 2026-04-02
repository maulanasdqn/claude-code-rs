use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::state::{DisplayToolUse, ToolUseStatus};

pub struct ToolProgress<'a> {
    pub tool_use: &'a DisplayToolUse,
    pub spinner_frame: usize,
}

impl<'a> ToolProgress<'a> {
    pub fn new(tool_use: &'a DisplayToolUse, spinner_frame: usize) -> Self {
        Self {
            tool_use,
            spinner_frame,
        }
    }
}

impl<'a> Widget for ToolProgress<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (indicator, color) = match self.tool_use.status {
            ToolUseStatus::Running => {
                let frames = [
                    '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}',
                    '\u{2826}', '\u{2827}', '\u{2807}', '\u{280F}',
                ];
                let ch = frames[self.spinner_frame % frames.len()];
                (ch, Color::Yellow)
            }
            ToolUseStatus::Completed => ('\u{2713}', Color::Green),
            ToolUseStatus::Error => ('\u{2717}', Color::Red),
        };

        let line = Line::from(vec![
            Span::styled(format!("{indicator} "), Style::default().fg(color)),
            Span::styled(&self.tool_use.name, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                &self.tool_use.output_preview,
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        Paragraph::new(line).render(area, buf);
    }
}
