use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;

pub struct Banner;

impl Banner {
    pub fn new() -> Self { Self }
}

impl Default for Banner {
    fn default() -> Self { Self::new() }
}

const BANNER_ART: &[&str] = &[
    r" ____   _____ __   __ _   _ __  __",
    r"/ ___| |_   _|\ \ / /| \ | |\ \/ /",
    r"\___ \   | |   \ V / |  \| | \  / ",
    r" ___) |  | |    | |  | |\  | /  \ ",
    r"|____/   |_|    |_|  |_| \_|/_/\_\",
    r"            c o d e               ",
];

impl Widget for Banner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![Line::from("")];
        for art_line in BANNER_ART {
            lines.push(Line::from(Span::styled(
                *art_line,
                Style::default().fg(theme::PINE()).add_modifier(Modifier::BOLD),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(theme::MUTED()),
        )));
        Paragraph::new(lines).alignment(Alignment::Center).render(area, buf);
    }
}
