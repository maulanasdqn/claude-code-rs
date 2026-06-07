use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Widget},
};

use crate::state::{ToastKind, ToastState};
use crate::theme;

pub struct ToastStack<'a> {
    pub state: &'a ToastState,
}

impl<'a> ToastStack<'a> {
    pub fn new(state: &'a ToastState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ToastStack<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.state.items.is_empty() {
            return;
        }

        // Noice-style notify cards: rounded, level-colored, stacked top-right.
        let max_box_w = 48u16.min(area.width.saturating_sub(2));
        if max_box_w < 14 {
            return;
        }
        let bg = theme::BACKGROUND_ELEMENT();
        let right_edge = area.x + area.width.saturating_sub(1);
        let mut y = area.y + 1;

        for toast in &self.state.items {
            let (icon, color) = match toast.kind {
                ToastKind::Info => ("\u{24D8}", theme::ACCENT()),    // ⓘ
                ToastKind::Success => ("\u{2713}", theme::SUCCESS()), // ✓
                ToastKind::Warning => ("\u{25B3}", theme::WARNING()), // △
                ToastKind::Error => ("\u{2717}", theme::ERROR()),     // ✗
            };

            // A "title\nbody" message renders the first line bold as a heading.
            let (title, body) = match toast.message.split_once('\n') {
                Some((t, b)) => (t.trim(), b.trim()),
                None => (toast.message.as_str(), ""),
            };

            // Inside the borders we reserve: "│ " + 2-col icon gutter + text + " │".
            let text_w = (max_box_w as usize).saturating_sub(6).max(1);
            let mut wrapped: Vec<(String, bool)> = Vec::new(); // (text, is_title)
            for line in wrap_plain(title, text_w) {
                wrapped.push((line, true));
            }
            if !body.is_empty() {
                for line in wrap_plain(body, text_w) {
                    wrapped.push((line, false));
                }
            }

            let longest = wrapped.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(0);
            let inner = longest + 2; // text + 2-col icon gutter
            let box_w = (inner + 4) as u16; // "│ " + inner + " │"
            let box_h = wrapped.len() as u16 + 2; // top + body + bottom

            if y + box_h > area.y + area.height {
                break;
            }

            let x = right_edge.saturating_sub(box_w);
            let border = Style::default().fg(color).bg(bg);
            let dash: String = "\u{2500}".repeat(inner + 2);

            // Top border.
            let top = Rect { x, y, width: box_w, height: 1 };
            Clear.render(top, buf);
            Paragraph::new(Line::from(Span::styled(format!("\u{256D}{dash}\u{256E}"), border)))
                .style(Style::default().bg(bg))
                .render(top, buf);

            // Body rows.
            for (i, (text, is_title)) in wrapped.iter().enumerate() {
                let row = Rect { x, y: y + 1 + i as u16, width: box_w, height: 1 };
                Clear.render(row, buf);
                let gutter = if i == 0 {
                    Span::styled(format!("{icon} "), border.add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("  ".to_string(), Style::default().bg(bg))
                };
                let text_style = if *is_title {
                    Style::default().fg(theme::TEXT()).bg(bg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::SUBTLE()).bg(bg)
                };
                let pad = longest.saturating_sub(text.chars().count());
                let line = Line::from(vec![
                    Span::styled("\u{2502} ".to_string(), border),
                    gutter,
                    Span::styled(text.clone(), text_style),
                    Span::styled(" ".repeat(pad), Style::default().bg(bg)),
                    Span::styled(" \u{2502}".to_string(), border),
                ]);
                Paragraph::new(line).style(Style::default().bg(bg)).render(row, buf);
            }

            // Bottom border.
            let bot = Rect { x, y: y + 1 + wrapped.len() as u16, width: box_w, height: 1 };
            Clear.render(bot, buf);
            Paragraph::new(Line::from(Span::styled(format!("\u{2570}{dash}\u{256F}"), border)))
                .style(Style::default().bg(bg))
                .render(bot, buf);

            y = bot.y + 2; // one-row gap between cards
        }
    }
}

/// Greedy word-wrap a plain string to `width` columns, hard-splitting any word
/// longer than the line.
fn wrap_plain(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();

    for word in s.split_whitespace() {
        if cur.is_empty() {
            if word.chars().count() > width {
                let mut chars: Vec<char> = word.chars().collect();
                while chars.len() > width {
                    lines.push(chars[..width].iter().collect());
                    chars.drain(..width);
                }
                cur = chars.into_iter().collect();
            } else {
                cur = word.to_string();
            }
        } else if cur.chars().count() + 1 + word.chars().count() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
    }
    if !cur.is_empty() || lines.is_empty() {
        lines.push(cur);
    }
    lines
}
