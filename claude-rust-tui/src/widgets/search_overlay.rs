use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};

pub struct SearchOverlay<'a> {
    pub query: &'a str,
    pub results: &'a [String],
    pub selected: usize,
}

impl<'a> SearchOverlay<'a> {
    pub fn new(query: &'a str, results: &'a [String], selected: usize) -> Self {
        Self {
            query,
            results,
            selected,
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

impl<'a> Widget for SearchOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_area = centered_rect(60, 60, area);

        Clear.render(dialog_area, buf);

        let block = Block::default()
            .title(" Search History ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

        // Search input
        let search_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let search_line = Line::from(vec![
            Span::styled("/ ", Style::default().fg(Color::Yellow)),
            Span::styled(self.query, Style::default().fg(Color::White)),
        ]);

        Paragraph::new(search_line)
            .block(search_block)
            .render(chunks[0], buf);

        // Results list
        let items: Vec<ListItem<'_>> = self
            .results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(result.as_str(), style)))
            })
            .collect();

        List::new(items).render(chunks[1], buf);
    }
}
