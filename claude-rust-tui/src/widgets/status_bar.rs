use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::widgets::spinner::FRAMES;
use crate::theme;

pub struct StatusBar<'a> {
    pub model: &'a str,
    pub mode: &'a str,
    pub cost: f64,
    pub git_branch: Option<&'a str>,
    pub is_streaming: bool,
    pub spinner_frame: usize,
}

impl<'a> StatusBar<'a> {
    pub fn new(model: &'a str, mode: &'a str, cost: f64, git_branch: Option<&'a str>, is_streaming: bool, spinner_frame: usize) -> Self {
        Self { model, mode, cost, git_branch, is_streaming, spinner_frame }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(theme::SURFACE));
        }

        let sep = Span::styled("  ·  ", Style::default().fg(theme::HL_HIGH).bg(theme::SURFACE));

        let mode_span = if self.is_streaming {
            let ch = FRAMES[self.spinner_frame % FRAMES.len()];
            Span::styled(format!("{ch} generating"), Style::default().fg(theme::IRIS).bg(theme::SURFACE).add_modifier(Modifier::ITALIC))
        } else {
            let (icon, color) = match self.mode {
                "Auto-accept" => ("⚡ ", theme::GOLD),
                "Plan"        => ("◆ ", theme::IRIS),
                "Bypass"      => ("⚠ ", theme::LOVE),
                _             => ("● ", theme::FOAM),
            };
            Span::styled(format!("{icon}{}", self.mode), Style::default().fg(color).bg(theme::SURFACE))
        };

        let mut spans = vec![
            Span::styled(" ", Style::default().bg(theme::SURFACE)),
            Span::styled(self.model, Style::default().fg(theme::FOAM).bg(theme::SURFACE).add_modifier(Modifier::BOLD)),
            sep.clone(),
            mode_span,
            sep.clone(),
            Span::styled(format!("${:.4}", self.cost), Style::default().fg(theme::IRIS).bg(theme::SURFACE)),
        ];

        if let Some(branch) = self.git_branch {
            spans.push(sep);
            spans.push(Span::styled(
                format!("\u{E0A0} {branch}"),
                Style::default().fg(theme::PINE).bg(theme::SURFACE),
            ));
        }

        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
