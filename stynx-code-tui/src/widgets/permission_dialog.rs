use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use crate::state::PermissionChoice;
use crate::theme;

pub struct PermissionDialog<'a> {
    pub tool_name: &'a str,
    pub description: &'a str,
    pub choice: PermissionChoice,
}

impl<'a> PermissionDialog<'a> {
    pub fn new(tool_name: &'a str, description: &'a str, choice: PermissionChoice) -> Self {
        Self { tool_name, description, choice }
    }

    fn rect(area: Rect) -> Rect {
        let h = 15.min(area.height.saturating_sub(2));
        let y = area.y + area.height.saturating_sub(h);
        Rect { x: area.x, y, width: area.width, height: h }
    }
}

impl<'a> Widget for PermissionDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = Self::rect(area);
        Clear.render(dialog, buf);

        for y in dialog.y..dialog.y + dialog.height {
            for x in dialog.x..dialog.x + dialog.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND_PANEL()));
            }
            buf[(dialog.x, y)].set_symbol("▏");
            buf[(dialog.x, y)].set_style(
                Style::default().fg(theme::WARNING()).bg(theme::BACKGROUND_PANEL()),
            );
        }

        let content = Rect {
            x: dialog.x + 2,
            y: dialog.y + 1,
            width: dialog.width.saturating_sub(4),
            height: dialog.height.saturating_sub(2),
        };

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(content);

        Paragraph::new(Line::from(vec![
            Span::styled("△ ", Style::default().fg(theme::WARNING()).bg(theme::BACKGROUND_PANEL())),
            Span::styled(
                "Permission required",
                Style::default()
                    .fg(theme::TEXT())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(theme::BACKGROUND_PANEL()))
        .render(chunks[0], buf);

        Paragraph::new(Line::from(vec![
            Span::styled(
                "Tool: ",
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
            ),
            Span::styled(
                self.tool_name.to_string(),
                Style::default()
                    .fg(theme::ACCENT())
                    .bg(theme::BACKGROUND_PANEL())
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(theme::BACKGROUND_PANEL()))
        .render(chunks[1], buf);

        Paragraph::new(self.description)
            .style(Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()))
            .wrap(Wrap { trim: false })
            .render(chunks[2], buf);

        let bar_area = chunks[3];
        for y in bar_area.y..bar_area.y + bar_area.height {
            for x in bar_area.x..bar_area.x + bar_area.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND_ELEMENT()));
            }
        }

        let opts = [
            (PermissionChoice::Once, "Allow once"),
            (PermissionChoice::Always, "Allow always"),
            (PermissionChoice::Reject, "Reject"),
        ];
        let mut spans: Vec<Span<'static>> = vec![Span::styled(
            " ",
            Style::default().bg(theme::BACKGROUND_ELEMENT()),
        )];
        for (i, (val, label)) in opts.iter().enumerate() {
            let selected = *val == self.choice;
            let (bg, fg) = if selected {
                (theme::WARNING(), theme::BACKGROUND())
            } else {
                (theme::BACKGROUND_MENU(), theme::TEXT_MUTED())
            };
            spans.push(Span::styled(
                format!(" {label} "),
                Style::default().fg(fg).bg(bg).add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ));
            if i < opts.len() - 1 {
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(theme::BACKGROUND_ELEMENT()),
                ));
            }
        }

        let hint = Line::from(vec![
            Span::styled(
                "←→ ",
                Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_ELEMENT()),
            ),
            Span::styled(
                "select  ",
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_ELEMENT()),
            ),
            Span::styled(
                "↵ ",
                Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_ELEMENT()),
            ),
            Span::styled(
                "confirm ",
                Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_ELEMENT()),
            ),
        ]);

        let buttons_line = Line::from(spans);
        let buttons_w = buttons_line.width() as u16;
        let hint_w = hint.width() as u16;

        Paragraph::new(buttons_line)
            .style(Style::default().bg(theme::BACKGROUND_ELEMENT()))
            .render(
                Rect {
                    x: bar_area.x,
                    y: bar_area.y + 1,
                    width: buttons_w.min(bar_area.width),
                    height: 1,
                },
                buf,
            );
        if bar_area.width > hint_w {
            Paragraph::new(hint)
                .style(Style::default().bg(theme::BACKGROUND_ELEMENT()))
                .render(
                    Rect {
                        x: bar_area.x + bar_area.width - hint_w,
                        y: bar_area.y + 1,
                        width: hint_w,
                        height: 1,
                    },
                    buf,
                );
        }
    }
}
