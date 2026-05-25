use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Widget},
};

use crate::state::{ToastKind, ToastState};
use crate::theme;

pub struct ToastStack<'a> {
    pub state: &'a ToastState,
}

impl<'a> ToastStack<'a> {
    pub fn new(state: &'a ToastState) -> Self { Self { state } }
}

impl<'a> Widget for ToastStack<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.state.items.is_empty() {
            return;
        }

        let max_w = 50.min(area.width.saturating_sub(4)) as usize;
        let mut y = area.y + 1;
        let right_edge = area.x + area.width.saturating_sub(2);

        for toast in &self.state.items {
            if y >= area.y + area.height { break; }

            let (icon, color) = match toast.kind {
                ToastKind::Info => ("ⓘ", theme::ACCENT()),
                ToastKind::Success => ("✓", theme::SUCCESS()),
                ToastKind::Warning => ("△", theme::WARNING()),
                ToastKind::Error => ("✗", theme::ERROR()),
            };

            let trimmed: String = if toast.message.chars().count() > max_w.saturating_sub(4) {
                let mut s: String = toast.message.chars().take(max_w.saturating_sub(5)).collect();
                s.push('…');
                s
            } else {
                toast.message.clone()
            };

            let text = format!(" {icon}  {trimmed} ");
            let width = text.chars().count() as u16;
            let x = right_edge.saturating_sub(width);

            let rect = Rect { x, y, width, height: 1 };
            Clear.render(rect, buf);

            let line = Line::from(vec![
                Span::styled(format!(" {icon}  "), Style::default().fg(color).bg(theme::BACKGROUND_ELEMENT()).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{trimmed} "), Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_ELEMENT())),
            ]);
            Paragraph::new(line)
                .style(Style::default().bg(theme::BACKGROUND_ELEMENT()))
                .render(rect, buf);

            y += 1;
        }
    }
}
