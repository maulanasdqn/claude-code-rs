use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct ThinkingPanel<'a> {
    pub text: &'a str,
    pub spinner_frame: usize,
}

impl<'a> ThinkingPanel<'a> {
    pub fn new(text: &'a str, spinner_frame: usize) -> Self {
        Self { text, spinner_frame }
    }
}

impl<'a> Widget for ThinkingPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.text.trim().is_empty() || area.height == 0 || area.width < 4 {
            return;
        }

        let frame = FRAMES[self.spinner_frame % FRAMES.len()];

        let visible_body = (area.height as usize).saturating_sub(1);
        let body_lines: Vec<&str> = {
            let mut v: Vec<&str> = self
                .text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();
            if v.len() > visible_body {
                v = v[v.len() - visible_body..].to_vec();
            }
            v
        };

        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf[(x, y)].set_style(Style::default().bg(theme::BACKGROUND()));
            }
        }

        let accent = theme::IRIS();
        let bar_col = theme::OVERLAY();

        for y in area.y..area.y + area.height {
            buf[(area.x, y)]
                .set_char('▌')
                .set_style(Style::default().fg(accent).bg(theme::BACKGROUND()));
        }

        // Content sits at `bar + space` (col 2), aligning with chat message bodies.
        let pad: u16 = 2;
        let inner_x = area.x + pad;
        let inner_width = area.width.saturating_sub(pad + 1) as usize;
        let _ = bar_col;

        let header = Line::from(vec![
            Span::styled(
                format!("{frame} "),
                Style::default()
                    .fg(accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "thinking",
                Style::default()
                    .fg(theme::SUBTLE())
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        let inner_area = Rect {
            x: inner_x,
            y: area.y,
            width: inner_width as u16,
            height: area.height,
        };

        let mut lines: Vec<Line<'static>> = vec![header];
        for raw in body_lines {
            lines.extend(wrap_thinking_line(raw.trim_end(), inner_width));
        }

        Paragraph::new(lines).render(inner_area, buf);
    }
}

/// Word-wrap a thinking body line to `width` columns with a hanging indent, so
/// continuation lines align under the first line's text instead of overflowing
/// to the left edge.
fn wrap_thinking_line(raw: &str, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let trimmed = raw.trim_start();
    let indent_w = raw.len() - trimmed.len();
    let base_indent = indent_w;

    let (prefix, body, prefix_style) = if let Some(rest) = trimmed.strip_prefix("### ") {
        ("### ".to_string(), rest, Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD))
    } else if let Some(rest) = trimmed.strip_prefix("## ") {
        ("## ".to_string(), rest, Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD))
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        ("# ".to_string(), rest, Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD))
    } else if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("• ".to_string(), rest, Style::default().fg(theme::SUBTLE()))
    } else if let Some((num, rest)) = split_numbered(trimmed) {
        (format!("{num}. "), rest, Style::default().fg(theme::SUBTLE()))
    } else {
        (String::new(), trimmed, Style::default())
    };

    // Continuation lines indent to the first line's text column (hanging indent).
    let hang = base_indent + prefix.chars().count();
    let avail = width.saturating_sub(hang).max(1);

    // Tokenize the styled body into words, collapsing runs of whitespace.
    let mut words: Vec<(String, Style)> = Vec::new();
    for span in parse_thinking_inline(body) {
        let style = span.style;
        for w in span.content.split(' ') {
            if !w.is_empty() {
                words.push((w.to_string(), style));
            }
        }
    }

    let new_line = |first: bool| -> Vec<Span<'static>> {
        if first {
            let mut v = vec![Span::styled(" ".repeat(base_indent), Style::default())];
            if !prefix.is_empty() {
                v.push(Span::styled(prefix.clone(), prefix_style));
            }
            v
        } else {
            vec![Span::styled(" ".repeat(hang), Style::default())]
        }
    };

    let mut out: Vec<Line<'static>> = Vec::new();
    let mut spans = new_line(true);
    let mut text_w = 0usize; // columns of wrappable text on the current line

    for (word, style) in words {
        let wlen = word.chars().count();
        let need = if text_w == 0 { wlen } else { wlen + 1 };
        if text_w > 0 && text_w + need > avail {
            out.push(Line::from(std::mem::take(&mut spans)));
            spans = new_line(false);
            text_w = 0;
        }
        if text_w > 0 {
            spans.push(Span::styled(" ".to_string(), Style::default()));
            text_w += 1;
        }
        spans.push(Span::styled(word, style));
        text_w += wlen;
    }
    out.push(Line::from(spans));
    out
}

fn split_numbered(s: &str) -> Option<(String, &str)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i == 0 || i >= bytes.len() { return None; }
    if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1] == b' ' {
        let num = s[..i].to_string();
        let rest = &s[i + 2..];
        Some((num, rest))
    } else {
        None
    }
}

fn parse_thinking_inline(text: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let base = Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC);
    let bold = Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD);
    let code = Style::default().fg(theme::FOAM()).add_modifier(Modifier::ITALIC);

    let flush = |buf: &mut String, spans: &mut Vec<Span<'static>>| {
        if !buf.is_empty() {
            spans.push(Span::styled(std::mem::take(buf), base));
        }
    };

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            flush(&mut buf, &mut spans);
            i += 2;
            let mut inner = String::new();
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                inner.push(chars[i]);
                i += 1;
            }
            spans.push(Span::styled(inner, bold));
            i += 2;
        } else if chars[i] == '`' {
            flush(&mut buf, &mut spans);
            i += 1;
            let mut inner = String::new();
            while i < chars.len() && chars[i] != '`' {
                inner.push(chars[i]);
                i += 1;
            }
            spans.push(Span::styled(inner, code));
            if i < chars.len() { i += 1; }
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    flush(&mut buf, &mut spans);
    spans
}
