use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

pub struct PermissionDialog<'a> {
    pub tool_name: &'a str,
    pub description: &'a str,
}

impl<'a> PermissionDialog<'a> {
    pub fn new(tool_name: &'a str, description: &'a str) -> Self {
        Self {
            tool_name,
            description,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

impl<'a> Widget for PermissionDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_area = centered_rect(50, 40, area);

        Clear.render(dialog_area, buf);

        let block = Block::default()
            .title(" Permission Required ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

        // Tool name
        let tool_line = Line::from(vec![
            Span::styled("Tool: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.tool_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        Paragraph::new(tool_line).render(chunks[0], buf);

        // Description
        let desc = Paragraph::new(self.description)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        desc.render(chunks[1], buf);

        // Action buttons
        let actions = Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("es  "),
            Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("o  "),
            Span::styled("[A]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("lways"),
        ]);
        Paragraph::new(actions)
            .alignment(Alignment::Center)
            .render(chunks[2], buf);
    }
}
