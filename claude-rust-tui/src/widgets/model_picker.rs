use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Widget},
};

pub struct ModelPicker<'a> {
    pub options: &'a [String],
    pub selected: usize,
}

impl<'a> ModelPicker<'a> {
    pub fn new(options: &'a [String], selected: usize) -> Self {
        Self { options, selected }
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

impl<'a> Widget for ModelPicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog_area = centered_rect(40, 50, area);

        Clear.render(dialog_area, buf);

        let block = Block::default()
            .title(" Select Model ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let items: Vec<ListItem<'_>> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let prefix = if i == self.selected { "> " } else { "  " };
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(format!("{prefix}{opt}"), style)))
            })
            .collect();

        let list = List::new(items).block(block);
        list.render(dialog_area, buf);
    }
}
