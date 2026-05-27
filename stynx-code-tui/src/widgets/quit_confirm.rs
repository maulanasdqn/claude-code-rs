use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Widget},
};

use crate::theme;

pub struct QuitConfirmDialog;

impl Widget for QuitConfirmDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let w: u16 = 36;
        let h: u16 = 4;
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let dialog = Rect { x, y, width: w.min(area.width), height: h };

        Clear.render(dialog, buf);

        for dy in 0..dialog.height {
            for dx in 0..dialog.width {
                buf[(dialog.x + dx, dialog.y + dy)]
                    .set_style(Style::default().bg(theme::BACKGROUND_PANEL()));
            }
            buf[(dialog.x, dialog.y + dy)]
                .set_char('▌')
                .set_style(Style::default().fg(theme::ERROR()).bg(theme::BACKGROUND_PANEL()));
        }

        let inner = Rect {
            x: dialog.x + 2,
            y: dialog.y + 1,
            width: dialog.width.saturating_sub(3),
            height: dialog.height.saturating_sub(2),
        };

        let lines = vec![
            Line::from(Span::styled(
                "Quit stynx?",
                Style::default().fg(theme::TEXT()).add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("y", Style::default().fg(theme::SUCCESS()).add_modifier(Modifier::BOLD)),
                Span::styled(" confirm  ", Style::default().fg(theme::MUTED())),
                Span::styled("any key", Style::default().fg(theme::ERROR()).add_modifier(Modifier::BOLD)),
                Span::styled(" cancel", Style::default().fg(theme::MUTED())),
            ]),
        ];

        Paragraph::new(lines).render(inner, buf);
    }
}
