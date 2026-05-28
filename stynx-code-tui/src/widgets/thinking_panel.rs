use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
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

        let pad: u16 = 1;
        let inner_x = area.x + pad;
        let inner_width = area.width.saturating_sub(pad + 1) as usize;
        let _ = bar_col;

        let header = Line::from(vec![
            Span::styled(
                format!("  {frame} "),
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
            lines.push(render_thinking_line(raw.trim_end()));
        }

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner_area, buf);
    }
}

fn render_thinking_line(raw: &str) -> Line<'static> {
    let trimmed = raw.trim_start();
    let indent_w = raw.len() - trimmed.len();
    let indent = " ".repeat(4 + indent_w);

    let (prefix, body, prefix_style) = if let Some(rest) = trimmed.strip_prefix("### ") {
        (
            "### ".to_string(),
            rest,
            Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
        )
    } else if let Some(rest) = trimmed.strip_prefix("## ") {
        (
            "## ".to_string(),
            rest,
            Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
        )
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        (
            "# ".to_string(),
            rest,
            Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::BOLD),
        )
    } else if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("• ".to_string(), rest, Style::default().fg(theme::SUBTLE()))
    } else if let Some((num, rest)) = split_numbered(trimmed) {
        (format!("{num}. "), rest, Style::default().fg(theme::SUBTLE()))
    } else {
        (String::new(), trimmed, Style::default())
    };

    let mut spans: Vec<Span<'static>> = vec![Span::styled(indent, Style::default())];
    if !prefix.is_empty() {
        spans.push(Span::styled(prefix, prefix_style));
    }
    spans.extend(parse_thinking_inline(body));
    Line::from(spans)
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
