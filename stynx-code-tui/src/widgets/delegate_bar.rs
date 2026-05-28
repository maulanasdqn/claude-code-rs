use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct DelegateBar<'a> {
    pub agents: &'a [(String, String)],
    pub spinner_frame: usize,
}

impl<'a> DelegateBar<'a> {
    pub fn new(agents: &'a [(String, String)], spinner_frame: usize) -> Self {
        Self { agents, spinner_frame }
    }
}

impl<'a> Widget for DelegateBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.agents.is_empty() { return; }
        let spinner = FRAMES.get(self.spinner_frame % FRAMES.len()).copied().unwrap_or('·');
        let lines: Vec<Line<'static>> = self.agents.iter().take(area.height as usize).map(|(label, summary)| {
            Line::from(vec![
                Span::styled(format!("  {spinner} "), Style::default().fg(theme::IRIS()).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{label}"), Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)),
                Span::styled("  ", Style::default()),
                Span::styled(summary.clone(), Style::default().fg(theme::SUBTLE())),
            ])
        }).collect();
        Paragraph::new(lines).render(area, buf);
    }
}
