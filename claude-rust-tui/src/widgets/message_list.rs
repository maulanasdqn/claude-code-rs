use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::state::{ConversationState, ToolUseStatus};
use crate::theme;

pub struct MessageList<'a> {
    pub state: &'a mut ConversationState,
}

impl<'a> MessageList<'a> {
    pub fn new(state: &'a mut ConversationState) -> Self { Self { state } }
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
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::ROSE).add_modifier(Modifier::BOLD)));
            i += 2;
        } else if chars[i] == '`' {
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != '`' { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::FOAM)));
            if i < chars.len() { i += 1; }
        } else if chars[i] == '*' || chars[i] == '_' {
            let delim = chars[i];
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != delim { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::SUBTLE).add_modifier(Modifier::ITALIC)));
            if i < chars.len() { i += 1; }
        } else {
            buf.push(chars[i]); i += 1;
        }
    }
    if !buf.is_empty() { spans.push(Span::styled(buf, Style::default().fg(theme::TEXT))); }
    spans
}

fn render_md_line(raw: &str, in_code: bool) -> Line<'static> {
    let trimmed = raw.trim_end();
    if in_code {
        return Line::from(Span::styled(format!("  {trimmed}"), Style::default().fg(theme::MUTED)));
    }
    if let Some(r) = trimmed.strip_prefix("### ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::FOAM).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("## ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::ROSE).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("# ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::GOLD).add_modifier(Modifier::BOLD)));
    }
    let (pre, body) = if let Some(r) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("  • ".to_string(), r)
    } else if let Some(r) = trimmed.strip_prefix("  - ").or_else(|| trimmed.strip_prefix("  * ")) {
        ("    ◦ ".to_string(), r)
    } else {
        ("  ".to_string(), trimmed)
    };
    let mut spans = vec![Span::styled(pre, Style::default().fg(theme::PINE))];
    spans.extend(parse_inline(body));
    Line::from(spans)
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Rect { y: area.y + 1, height: area.height.saturating_sub(1), ..area };
        let mut lines: Vec<Line<'static>> = Vec::new();

        for msg in &self.state.messages {
            match msg.role.as_str() {
                "user" => {
                    for (i, raw) in msg.content.lines().enumerate() {
                        let line = if i == 0 {
                            Line::from(vec![
                                Span::styled("  ❯ ", Style::default().fg(theme::FOAM).add_modifier(Modifier::BOLD)),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::TEXT)),
                            ])
                        } else {
                            Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::TEXT)))
                        };
                        lines.push(line);
                    }
                }
                "error" => {
                    for (i, raw) in msg.content.lines().enumerate() {
                        let line = if i == 0 {
                            Line::from(vec![
                                Span::styled("  ✗ ", Style::default().fg(theme::LOVE).add_modifier(Modifier::BOLD)),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::LOVE)),
                            ])
                        } else {
                            Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::LOVE)))
                        };
                        lines.push(line);
                    }
                }
                "system" => {
                    for raw in msg.content.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  · {}", raw.trim_end()),
                            Style::default().fg(theme::MUTED).add_modifier(Modifier::ITALIC),
                        )));
                    }
                }
                _ => {
                    if !msg.thinking.is_empty() {
                        lines.push(Line::from(Span::styled("  ◈ thinking", Style::default().fg(theme::MUTED).add_modifier(Modifier::ITALIC))));
                        for raw in msg.thinking.lines() {
                            lines.push(Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::MUTED).add_modifier(Modifier::DIM))));
                        }
                    }
                    let mut in_code = false;
                    let mut in_mermaid = false;
                    for raw in msg.content.lines() {
                        let trimmed_start = raw.trim_start();
                        if trimmed_start.starts_with("```mermaid") {
                            in_mermaid = true; in_code = true;
                            lines.push(Line::from(Span::styled("  ╭─ mermaid ", Style::default().fg(theme::IRIS).add_modifier(Modifier::BOLD))));
                        } else if in_mermaid && trimmed_start.starts_with("```") {
                            in_mermaid = false; in_code = false;
                            lines.push(Line::from(Span::styled("  ╰────────── ", Style::default().fg(theme::IRIS))));
                        } else if in_mermaid {
                            lines.push(Line::from(vec![
                                Span::styled("  │ ", Style::default().fg(theme::IRIS)),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::GOLD)),
                            ]));
                        } else if trimmed_start.starts_with("```") {
                            in_code = !in_code;
                            lines.push(Line::from(Span::styled(format!("  {}", raw.trim_end()), Style::default().fg(theme::OVERLAY))));
                        } else {
                            lines.push(render_md_line(raw, in_code));
                        }
                    }
                    let _ = in_code;
                    for tool in &msg.tool_uses {
                        let (icon, col) = match tool.status {
                            ToolUseStatus::Running   => ("◌", theme::GOLD),
                            ToolUseStatus::Completed => ("✓", theme::FOAM),
                            ToolUseStatus::Error     => ("✗", theme::LOVE),
                        };
                        let preview = if tool.output_preview.is_empty() { String::new() }
                            else { format!("  {}", tool.output_preview) };
                        lines.push(Line::from(vec![
                            Span::styled(format!("  {icon} "), Style::default().fg(col)),
                            Span::styled(tool.name.clone(), Style::default().fg(theme::SUBTLE)),
                            Span::styled(preview, Style::default().fg(theme::MUTED).add_modifier(Modifier::DIM)),
                        ]));
                    }
                }
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        let total = lines.len();
        let visible = area.height as usize;
        self.state.total_lines = total;
        let offset = if self.state.auto_scroll {
            let off = total.saturating_sub(visible);
            self.state.scroll_offset = off;
            off
        } else {
            self.state.scroll_offset
        };

        Paragraph::new(lines).scroll((offset as u16, 0)).wrap(Wrap { trim: false }).render(area, buf);
    }
}
