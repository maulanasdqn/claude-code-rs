use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::state::{DiffLineKind, DisplayToolUse, ToolUseStatus};
use crate::theme;

pub struct ToolDetail<'a> {
    pub tool: &'a DisplayToolUse,
}

impl<'a> ToolDetail<'a> {
    pub fn new(tool: &'a DisplayToolUse) -> Self {
        Self { tool }
    }
}

fn status_label(s: ToolUseStatus) -> (&'static str, ratatui::style::Color) {
    match s {
        ToolUseStatus::Running => ("running", theme::GOLD()),
        ToolUseStatus::Completed => ("done", theme::SUCCESS()),
        ToolUseStatus::Error => ("error", theme::ERROR()),
    }
}

impl<'a> Widget for ToolDetail<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_w = area.width.saturating_sub(8).min(160);
        let popup_h = area.height.saturating_sub(4).min(40);
        let popup = Rect {
            x: area.x + (area.width.saturating_sub(popup_w)) / 2,
            y: area.y + (area.height.saturating_sub(popup_h)) / 2,
            width: popup_w,
            height: popup_h,
        };

        Clear.render(popup, buf);

        let (status_text, status_col) = status_label(self.tool.status);
        let title_line = Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(self.tool.name.clone(), Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(status_text, Style::default().fg(status_col).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default()),
        ]);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title_line)
            .border_style(Style::default().fg(theme::IRIS()))
            .style(Style::default().bg(theme::BACKGROUND()));

        let inner = block.inner(popup);
        block.render(popup, buf);

        let body = inner.inner(Margin { vertical: 1, horizontal: 2 });

        let mut lines: Vec<Line<'static>> = Vec::new();

        if !self.tool.input_summary.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("input  ", Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD)),
                Span::styled(self.tool.input_summary.clone(), Style::default().fg(theme::TEXT())),
            ]));
            lines.push(Line::from(""));
        }

        if !self.tool.diff.is_empty() {
            lines.push(Line::from(Span::styled(
                "diff",
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
            )));
            for d in &self.tool.diff {
                let (sign, sign_col, body_col) = match d.kind {
                    DiffLineKind::Added => ("+", theme::SUCCESS(), theme::TEXT()),
                    DiffLineKind::Removed => ("-", theme::ERROR(), theme::TEXT()),
                    DiffLineKind::Context => (" ", theme::TEXT_MUTED(), theme::TEXT_MUTED()),
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{sign} "), Style::default().fg(sign_col).add_modifier(Modifier::BOLD)),
                    Span::styled(d.text.clone(), Style::default().fg(body_col)),
                ]));
            }
            lines.push(Line::from(""));
        }

        if !self.tool.output_excerpt.is_empty() {
            lines.push(Line::from(Span::styled(
                "output",
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
            )));
            for l in &self.tool.output_excerpt {
                lines.push(Line::from(Span::styled(l.clone(), Style::default().fg(theme::TEXT_MUTED()))));
            }
            lines.push(Line::from(""));
        } else if !self.tool.output_preview.is_empty() {
            lines.push(Line::from(Span::styled(
                "output",
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                self.tool.output_preview.clone(),
                Style::default().fg(theme::TEXT_MUTED()),
            )));
            lines.push(Line::from(""));
        }

        if !self.tool.sub_progress.is_empty() {
            lines.push(Line::from(Span::styled(
                "delegated steps",
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
            )));
            for l in &self.tool.sub_progress {
                lines.push(Line::from(vec![
                    Span::styled("↪ ", Style::default().fg(theme::IRIS())),
                    Span::styled(l.clone(), Style::default().fg(theme::TEXT_MUTED())),
                ]));
            }
        }

        let footer = Line::from(Span::styled(
            "  esc to close",
            Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::ITALIC | Modifier::DIM),
        ));
        lines.push(Line::from(""));
        lines.push(footer);

        Paragraph::new(lines).wrap(Wrap { trim: false }).render(body, buf);
    }
}
