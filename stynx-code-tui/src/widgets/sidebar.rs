use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::state::app_state::SidebarState;
use crate::theme;

pub struct Sidebar<'a> {
    state: &'a SidebarState,
}

impl<'a> Sidebar<'a> {
    pub fn new(state: &'a SidebarState) -> Self { Self { state } }
}

impl<'a> Widget for Sidebar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND_PANEL()));
            }
        }

        let inner = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };

        let mut lines: Vec<Line<'static>> = Vec::new();

        lines.push(Line::from(Span::styled(
            self.state.title.clone(),
            Style::default()
                .fg(theme::TEXT())
                .bg(theme::BACKGROUND_PANEL())
                .add_modifier(Modifier::BOLD),
        )));
        if !self.state.session_id.is_empty() {
            lines.push(Line::from(Span::styled(
                self.state.session_id.clone(),
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
            )));
        }
        lines.push(Line::from(""));

        if !self.state.sessions.is_empty() {
            lines.push(Line::from(Span::styled(
                "Recent",
                Style::default()
                    .fg(theme::TEXT_MUTED())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::DIM),
            )));
            for s in self.state.sessions.iter().take(20) {
                let prefix = if s.pinned { "★ " } else { "  " };
                let title = if s.title.len() > 32 { format!("{}…", &s.title[..31]) } else { s.title.clone() };
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::WARNING()).bg(theme::BACKGROUND_PANEL())),
                    Span::styled(title, Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL())),
                ]));
            }
        }

        let body_height = inner.height.saturating_sub(2);
        let body_area = Rect { height: body_height, ..inner };
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(body_area, buf);

        let footer_y = inner.y + inner.height.saturating_sub(1);
        let footer_area = Rect { x: inner.x, y: footer_y, width: inner.width, height: 1 };
        let footer = Line::from(vec![
            Span::styled("• ", Style::default().fg(theme::SUCCESS()).bg(theme::BACKGROUND_PANEL())),
            Span::styled("stynx", Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL())),
            Span::styled(
                "-code",
                Style::default()
                    .fg(theme::TEXT())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" v{}", self.state.version),
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
            ),
        ]);
        Paragraph::new(footer)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(footer_area, buf);
    }
}
