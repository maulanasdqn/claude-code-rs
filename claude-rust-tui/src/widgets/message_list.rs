use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::state::{ConversationState, ToolUseStatus};

pub struct MessageList<'a> {
    pub state: &'a ConversationState,
}

impl<'a> MessageList<'a> {
    pub fn new(state: &'a ConversationState) -> Self { Self { state } }
}

fn parse_inline(text: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '*' && chars[i+1] == '*' {
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 2;
            while i < chars.len() && !(i + 1 < chars.len() && chars[i] == '*' && chars[i+1] == '*') {
                buf.push(chars[i]); i += 1;
            }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().add_modifier(Modifier::BOLD)));
            i += 2;
        } else if chars[i] == '`' {
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != '`' { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(Color::Cyan)));
            if i < chars.len() { i += 1; }
        } else if chars[i] == '*' || chars[i] == '_' {
            let delim = chars[i];
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != delim { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().add_modifier(Modifier::ITALIC)));
            if i < chars.len() { i += 1; }
        } else {
            buf.push(chars[i]); i += 1;
        }
    }
    if !buf.is_empty() { spans.push(Span::raw(buf)); }
    spans
}

fn render_md_line(raw: &str, in_code: bool) -> Line<'static> {
    let trimmed = raw.trim_end();
    if in_code {
        return Line::from(Span::styled(format!("  {trimmed}"), Style::default().fg(Color::DarkGray)));
    }
    if let Some(r) = trimmed.strip_prefix("### ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("## ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("# ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
    }
    let (pre, body) = if let Some(r) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("  • ".to_string(), r)
    } else if let Some(r) = trimmed.strip_prefix("  - ").or_else(|| trimmed.strip_prefix("  * ")) {
        ("    ◦ ".to_string(), r)
    } else {
        ("  ".to_string(), trimmed)
    };
    let mut spans = vec![Span::raw(pre)];
    spans.extend(parse_inline(body));
    Line::from(spans)
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        for msg in &self.state.messages {
            let (label, color) = match msg.role.as_str() {
                "user"      => ("You", Color::Green),
                "assistant" => ("Assistant", Color::Blue),
                "error"     => ("Error", Color::Red),
                _           => ("System", Color::DarkGray),
            };
            lines.push(Line::from(Span::styled(
                format!("  {label}"),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));

            if !msg.thinking.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  ◈ Thinking",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                )));
                for raw in msg.thinking.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", raw.trim_end()),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
                    )));
                }
            }

            let mut in_code = false;
            for raw in msg.content.lines() {
                let is_fence = raw.trim_start().starts_with("```");
                if is_fence {
                    in_code = !in_code;
                    lines.push(Line::from(Span::styled(
                        format!("  {}", raw.trim_end()),
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    lines.push(render_md_line(raw, in_code));
                }
            }
            in_code = false;

            for tool in &msg.tool_uses {
                let (icon, col) = match tool.status {
                    ToolUseStatus::Running   => ("◌", Color::Yellow),
                    ToolUseStatus::Completed => ("✓", Color::Green),
                    ToolUseStatus::Error     => ("✗", Color::Red),
                };
                let preview = if tool.output_preview.is_empty() { String::new() }
                    else { format!("  {}", tool.output_preview) };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {icon} "), Style::default().fg(col)),
                    Span::styled(tool.name.clone(), Style::default().fg(Color::DarkGray)),
                    Span::styled(preview, Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
                ]));
                let _ = in_code;
            }

            lines.push(Line::from(""));
        }

        let total = lines.len();
        let visible = area.height as usize;
        let offset = if self.state.auto_scroll { total.saturating_sub(visible) } else { self.state.scroll_offset };

        Paragraph::new(lines).scroll((offset as u16, 0)).wrap(Wrap { trim: false }).render(area, buf);
    }
}
