use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

pub struct AutocompleteDropdown<'a> {
    pub options: &'a [String],
    pub selected: usize,
}

impl<'a> AutocompleteDropdown<'a> {
    pub fn new(options: &'a [String], selected: usize) -> Self {
        Self { options, selected }
    }
}

impl<'a> Widget for AutocompleteDropdown<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem<'_>> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(opt.as_str(), style)))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let list = List::new(items).block(block);
        list.render(area, buf);
    }
}
