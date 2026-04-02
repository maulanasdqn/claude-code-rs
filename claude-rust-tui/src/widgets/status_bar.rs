use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;

pub struct StatusBar<'a> {
    pub model: &'a str,
    pub mode: &'a str,
    pub cost: f64,
    pub git_branch: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(model: &'a str, mode: &'a str, cost: f64, git_branch: Option<&'a str>) -> Self {
        Self { model, mode, cost, git_branch }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(theme::SURFACE));
        }

        let sep = Span::styled("  ·  ", Style::default().fg(theme::HL_HIGH).bg(theme::SURFACE));

        let (mode_icon, mode_color) = match self.mode {
            "Auto-accept" => ("⚡ ", theme::GOLD),
            "Plan"        => ("◆ ", theme::IRIS),
            "Bypass"      => ("⚠ ", theme::LOVE),
            _             => ("● ", theme::FOAM),
        };

        let mut spans = vec![
            Span::styled(" ", Style::default().bg(theme::SURFACE)),
            Span::styled(self.model, Style::default().fg(theme::FOAM).bg(theme::SURFACE).add_modifier(Modifier::BOLD)),
            sep.clone(),
            Span::styled(mode_icon, Style::default().fg(mode_color).bg(theme::SURFACE).add_modifier(Modifier::BOLD)),
            Span::styled(self.mode, Style::default().fg(mode_color).bg(theme::SURFACE)),
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
