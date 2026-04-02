use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Banner;

impl Banner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Banner {
    fn default() -> Self {
        Self::new()
    }
}

const BANNER_ART: &[&str] = &[
    r"      _                 _                          _   ",
    r"  ___| | __ _ _   _  __| | ___       _ __ _   _ ___| |_ ",
    r" / __| |/ _` | | | |/ _` |/ _ \___  | '__| | | / __| __|",
    r"| (__| | (_| | |_| | (_| |  __/___| | |  | |_| \__ \ |_ ",
    r" \___|_|\__,_|\__,_|\__,_|\___|     |_|   \__,_|___/\__|",
];

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line<'_>> = Vec::new();

        // Blank line before art
        lines.push(Line::from(""));

        for art_line in BANNER_ART {
            lines.push(Line::from(Span::styled(
                *art_line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        // Version line
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        )));

        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
