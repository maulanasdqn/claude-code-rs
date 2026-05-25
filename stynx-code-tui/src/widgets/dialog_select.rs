use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::state::{DialogOption, filter_options};
use crate::theme;

pub struct DialogSelect<'a> {
    pub title: &'a str,
    pub query: &'a str,
    pub options: &'a [DialogOption],
    pub selected: usize,
    pub current_value: Option<&'a str>,
    pub footer_hint: Option<&'a str>,
}

impl<'a> DialogSelect<'a> {
    pub fn new(
        title: &'a str,
        query: &'a str,
        options: &'a [DialogOption],
        selected: usize,
    ) -> Self {
        Self {
            title,
            query,
            options,
            selected,
            current_value: None,
            footer_hint: None,
        }
    }

    pub fn with_current(mut self, v: Option<&'a str>) -> Self {
        self.current_value = v;
        self
    }

    pub fn with_footer(mut self, h: Option<&'a str>) -> Self {
        self.footer_hint = h;
        self
    }

    fn dialog_rect(area: Rect) -> Rect {
        let target_w = 78.min(area.width.saturating_sub(4));
        let target_h = 22.min(area.height.saturating_sub(4));
        let x = area.x + (area.width.saturating_sub(target_w)) / 2;
        let y = area.y + (area.height.saturating_sub(target_h)) / 2;
        Rect { x, y, width: target_w, height: target_h }
    }
}

impl<'a> Widget for DialogSelect<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dialog = Self::dialog_rect(area);
        Clear.render(dialog, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BORDER_ACTIVE()))
            .style(Style::default().bg(theme::BACKGROUND_PANEL()));
        let inner = block.inner(dialog);
        block.render(dialog, buf);

        let rows = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Length(1), // search input
            Constraint::Length(1), // divider
            Constraint::Min(1),    // options
            Constraint::Length(1), // footer
        ])
        .split(inner);

        Paragraph::new(Line::from(Span::styled(
            format!(" {} ", self.title),
            Style::default()
                .fg(theme::TEXT())
                .bg(theme::BACKGROUND_PANEL())
                .add_modifier(Modifier::BOLD),
        )))
        .style(Style::default().bg(theme::BACKGROUND_PANEL()))
        .render(rows[0], buf);

        let query_display = if self.query.is_empty() {
            Line::from(vec![
                Span::styled("  › ", Style::default().fg(theme::ACCENT()).bg(theme::BACKGROUND_PANEL())),
                Span::styled(
                    "type to filter…",
                    Style::default()
                        .fg(theme::TEXT_MUTED())
                        .bg(theme::BACKGROUND_PANEL())
                        .add_modifier(Modifier::ITALIC),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("  › ", Style::default().fg(theme::ACCENT()).bg(theme::BACKGROUND_PANEL())),
                Span::styled(
                    self.query.to_string(),
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled("▏", Style::default().fg(theme::ACCENT()).bg(theme::BACKGROUND_PANEL())),
            ])
        };
        Paragraph::new(query_display)
            .style(Style::default().bg(theme::BACKGROUND_PANEL()))
            .render(rows[1], buf);

        let divider_line = "─".repeat(rows[2].width as usize);
        Paragraph::new(Line::from(Span::styled(
            divider_line,
            Style::default().fg(theme::BORDER()).bg(theme::BACKGROUND_PANEL()),
        )))
        .render(rows[2], buf);

        let list_area = rows[3];
        let filtered = filter_options(self.options, self.query);
        let visible = list_area.height as usize;
        let selected_clamped = self.selected.min(filtered.len().saturating_sub(1));
        let scroll = if filtered.len() <= visible {
            0
        } else if selected_clamped < visible / 2 {
            0
        } else if selected_clamped + visible / 2 >= filtered.len() {
            filtered.len() - visible
        } else {
            selected_clamped - visible / 2
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut last_category: Option<String> = None;
        for (i, &orig_idx) in filtered.iter().enumerate().skip(scroll).take(visible) {
            let opt = &self.options[orig_idx];
            if opt.category != last_category {
                if let Some(cat) = &opt.category {
                    lines.push(Line::from(Span::styled(
                        format!("  {cat}"),
                        Style::default()
                            .fg(theme::TEXT_MUTED())
                            .bg(theme::BACKGROUND_PANEL())
                            .add_modifier(Modifier::DIM | Modifier::ITALIC),
                    )));
                }
                last_category = opt.category.clone();
            }

            let is_selected = i == selected_clamped;
            let is_current = self
                .current_value
                .map(|c| c == opt.value.as_str())
                .unwrap_or(false);

            let row_bg = if is_selected {
                theme::BACKGROUND_MENU()
            } else {
                theme::BACKGROUND_PANEL()
            };
            let arrow_span = if is_selected {
                Span::styled(" ▶ ", Style::default().fg(theme::ACCENT()).bg(row_bg))
            } else if is_current {
                Span::styled(" • ", Style::default().fg(theme::PRIMARY()).bg(row_bg))
            } else {
                Span::styled("   ", Style::default().bg(row_bg))
            };
            let title_style = if opt.disabled {
                Style::default()
                    .fg(theme::TEXT_MUTED())
                    .bg(row_bg)
                    .add_modifier(Modifier::DIM)
            } else if is_selected {
                Style::default()
                    .fg(theme::TEXT())
                    .bg(row_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT()).bg(row_bg)
            };

            let mut spans = vec![arrow_span, Span::styled(opt.title.clone(), title_style)];
            if let Some(desc) = &opt.description {
                spans.push(Span::styled(
                    format!("  {desc}"),
                    Style::default()
                        .fg(theme::TEXT_MUTED())
                        .bg(row_bg)
                        .add_modifier(Modifier::DIM),
                ));
            }
            if let Some(footer) = &opt.footer {
                spans.push(Span::styled(
                    format!("  {footer}"),
                    Style::default().fg(theme::SUCCESS()).bg(row_bg),
                ));
            }
            let line = Line::from(spans);
            let line_width = line.width() as u16;
            let row_y = list_area.y + (i - scroll) as u16;
            if row_y >= list_area.y + list_area.height {
                break;
            }
            for x in list_area.x..list_area.x + list_area.width {
                buf[(x, row_y)].set_style(Style::default().bg(row_bg));
            }
            let row_rect = Rect {
                x: list_area.x,
                y: row_y,
                width: line_width.min(list_area.width),
                height: 1,
            };
            Paragraph::new(line).render(row_rect, buf);
        }

        let _ = lines;

        let footer = match self.footer_hint {
            Some(h) => Line::from(vec![
                Span::styled(
                    " ↑↓ ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "navigate ",
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "↵ ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "select ",
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "esc ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    format!("close   {h}"),
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
            ]),
            None => Line::from(vec![
                Span::styled(
                    " ↑↓ ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "navigate ",
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "↵ ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "select ",
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "esc ",
                    Style::default().fg(theme::TEXT()).bg(theme::BACKGROUND_PANEL()),
                ),
                Span::styled(
                    "close",
                    Style::default().fg(theme::TEXT_MUTED()).bg(theme::BACKGROUND_PANEL()),
                ),
            ]),
        };
        for x in rows[4].x..rows[4].x + rows[4].width {
            buf[(x, rows[4].y)].set_style(Style::default().bg(theme::BACKGROUND_ELEMENT()));
        }
        Paragraph::new(footer)
            .style(Style::default().bg(theme::BACKGROUND_ELEMENT()))
            .render(rows[4], buf);
    }
}
