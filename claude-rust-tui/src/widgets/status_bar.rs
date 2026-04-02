use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct StatusBar<'a> {
    pub model: &'a str,
    pub mode: &'a str,
    pub cost: f64,
    pub git_branch: Option<&'a str>,
}

impl<'a> StatusBar<'a> {
    pub fn new(model: &'a str, mode: &'a str, cost: f64, git_branch: Option<&'a str>) -> Self {
        Self {
            model,
            mode,
            cost,
            git_branch,
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill background
        for x in area.x..area.x + area.width {
            buf[(x, area.y)].set_style(Style::default().bg(Color::DarkGray));
        }

        let mut spans = vec![
            Span::styled(
                format!(" {} ", self.model),
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " | ",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
            Span::styled(
                self.mode,
                Style::default().fg(Color::Yellow).bg(Color::DarkGray),
            ),
            Span::styled(
                " | ",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
            Span::styled(
                format!("${:.4}", self.cost),
                Style::default().fg(Color::Green).bg(Color::DarkGray),
            ),
        ];

        if let Some(branch) = self.git_branch {
            spans.push(Span::styled(
                " | ",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ));
            spans.push(Span::styled(
                format!("\u{E0A0} {branch}"),
                Style::default().fg(Color::Magenta).bg(Color::DarkGray),
            ));
        }

        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
