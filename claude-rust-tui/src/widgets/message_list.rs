use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::state::ConversationState;

pub struct MessageList<'a> {
    pub state: &'a ConversationState,
}

impl<'a> MessageList<'a> {
    pub fn new(state: &'a ConversationState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line<'_>> = Vec::new();

        for msg in &self.state.messages {
            let (prefix, color) = match msg.role.as_str() {
                "user" => ("> ", Color::Green),
                "assistant" => ("  ", Color::Blue),
                _ => ("  ", Color::White),
            };

            // Role header
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, msg.role),
                Style::default().fg(color),
            )));

            // Content lines
            for content_line in msg.content.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  {content_line}"),
                    Style::default().fg(Color::White),
                )));
            }

            // Tool uses
            for tool in &msg.tool_uses {
                let status_char = match tool.status {
                    crate::state::ToolUseStatus::Running => '\u{25CB}',   // circle
                    crate::state::ToolUseStatus::Completed => '\u{2713}', // checkmark
                    crate::state::ToolUseStatus::Error => '\u{2717}',     // x mark
                };
                lines.push(Line::from(Span::styled(
                    format!("  {status_char} {}", tool.name),
                    Style::default().fg(Color::Yellow),
                )));
            }

            // Blank separator
            lines.push(Line::from(""));
        }

        let total_lines = lines.len();
        let visible = area.height as usize;
        let offset = if self.state.auto_scroll {
            total_lines.saturating_sub(visible)
        } else {
            self.state.scroll_offset
        };

        let paragraph = Paragraph::new(lines)
            .scroll((offset as u16, 0))
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}
