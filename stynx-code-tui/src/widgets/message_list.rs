use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use ratatui::layout::Alignment;

use crate::state::{ConversationState, DiffLineKind, ToolUseStatus};
use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct MessageList<'a> {
    pub state: &'a mut ConversationState,
    pub spinner_frame: usize,
    pub tool_details: bool,
}

impl<'a> MessageList<'a> {
    pub fn new(state: &'a mut ConversationState, spinner_frame: usize) -> Self {
        Self { state, spinner_frame, tool_details: true }
    }

    pub fn with_tool_details(mut self, on: bool) -> Self {
        self.tool_details = on;
        self
    }
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
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::ROSE()).add_modifier(Modifier::BOLD)));
            i += 2;
        } else if chars[i] == '`' {
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != '`' { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)));
            if i < chars.len() { i += 1; }
        } else if chars[i] == '*' || chars[i] == '_' {
            let delim = chars[i];
            if !buf.is_empty() { spans.push(Span::raw(std::mem::take(&mut buf))); }
            i += 1;
            while i < chars.len() && chars[i] != delim { buf.push(chars[i]); i += 1; }
            spans.push(Span::styled(std::mem::take(&mut buf), Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::ITALIC)));
            if i < chars.len() { i += 1; }
        } else {
            buf.push(chars[i]); i += 1;
        }
    }
    if !buf.is_empty() { spans.push(Span::styled(buf, Style::default().fg(theme::TEXT()))); }
    spans
}

fn is_hr(s: &str) -> bool {
    let t = s.trim();
    (t.starts_with("---") || t.starts_with("===") || t.starts_with("***"))
        && t.chars().collect::<std::collections::HashSet<_>>().len() == 1
}

fn is_table_row(s: &str) -> bool {
    let t = s.trim();
    t.starts_with('|') && t.ends_with('|')
}

fn is_table_sep(s: &str) -> bool {
    let t = s.trim();
    is_table_row(t) && t.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
}

fn render_table_row(raw: &str) -> Line<'static> {
    let cells: Vec<&str> = raw.trim().trim_matches('|').split('|').collect();
    let mut spans = vec![Span::styled("  ", Style::default())];
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme::OVERLAY())));
        }
        let trimmed = cell.trim().to_string();
        spans.push(Span::styled(trimmed, Style::default().fg(theme::TEXT())));
    }
    Line::from(spans)
}

fn render_md_line(raw: &str, in_code: bool) -> Line<'static> {
    let trimmed = raw.trim_end();
    if in_code {
        return Line::from(vec![
            Span::styled("  │ ", Style::default().fg(theme::OVERLAY())),
            Span::styled(trimmed.to_string(), Style::default().fg(theme::GOLD())),
        ]);
    }
    if is_hr(trimmed) {
        return Line::from(Span::styled(
            "  ────────────────────────────────────────",
            Style::default().fg(theme::OVERLAY()),
        ));
    }
    if is_table_sep(trimmed) {
        return Line::from(Span::styled(
            "  ─────────────────────────────────────────",
            Style::default().fg(theme::OVERLAY()).add_modifier(Modifier::DIM),
        ));
    }
    if is_table_row(trimmed) {
        return render_table_row(trimmed);
    }
    if let Some(r) = trimmed.strip_prefix("### ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("## ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::ROSE()).add_modifier(Modifier::BOLD)));
    }
    if let Some(r) = trimmed.strip_prefix("# ") {
        return Line::from(Span::styled(format!("  {r}"), Style::default().fg(theme::GOLD()).add_modifier(Modifier::BOLD)));
    }
    let (pre, body) = if let Some(r) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("  • ".to_string(), r)
    } else if let Some(r) = trimmed.strip_prefix("  - ").or_else(|| trimmed.strip_prefix("  * ")) {
        ("    ◦ ".to_string(), r)
    } else if let Some(r) = trimmed.strip_prefix("    - ").or_else(|| trimmed.strip_prefix("    * ")) {
        ("      · ".to_string(), r)
    } else {
        ("  ".to_string(), trimmed)
    };
    let mut spans = vec![Span::styled(pre, Style::default().fg(theme::PINE()))];
    spans.extend(parse_inline(body));
    Line::from(spans)
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Rect { y: area.y + 1, height: area.height.saturating_sub(1), ..area };

        if self.state.messages.is_empty() {
            draw_empty_state(area, buf);
            return;
        }

        let mut lines: Vec<Line<'static>> = Vec::new();

        for msg in &self.state.messages {
            match msg.role.as_str() {
                "user" => {
                    for (i, raw) in msg.content.lines().enumerate() {
                        let line = if i == 0 {
                            Line::from(vec![
                                Span::styled("  ❯ ", Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::TEXT())),
                            ])
                        } else {
                            Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::TEXT())))
                        };
                        lines.push(line);
                    }
                }
                "error" => {
                    for (i, raw) in msg.content.lines().enumerate() {
                        let line = if i == 0 {
                            Line::from(vec![
                                Span::styled("  ✗ ", Style::default().fg(theme::LOVE()).add_modifier(Modifier::BOLD)),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::LOVE())),
                            ])
                        } else {
                            Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::LOVE())))
                        };
                        lines.push(line);
                    }
                }
                "system" => {
                    for raw in msg.content.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  · {}", raw.trim_end()),
                            Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC),
                        )));
                    }
                }
                _ => {
                    if !msg.thinking.is_empty() {
                        let think_lines: Vec<&str> = msg.thinking.lines().collect();
                        let show_lines = if msg.is_streaming {
                            &think_lines[think_lines.len().saturating_sub(3)..]
                        } else {
                            &think_lines[..]
                        };
                        let label = if msg.is_streaming { "  ◈ thinking..." } else { "  ◈ thinking" };
                        lines.push(Line::from(Span::styled(label, Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC))));
                        for raw in show_lines {
                            lines.push(Line::from(Span::styled(format!("    {}", raw.trim_end()), Style::default().fg(theme::MUTED()).add_modifier(Modifier::DIM))));
                        }
                    }
                    let mut in_code = false;
                    let mut in_mermaid = false;
                    let mut prev_blank = false;
                    for raw in msg.content.lines() {
                        let trimmed_start = raw.trim_start();
                        let is_blank = raw.trim().is_empty();

                        if is_blank {
                            if !prev_blank { lines.push(Line::from("")); }
                            prev_blank = true;
                            continue;
                        }
                        prev_blank = false;

                        if trimmed_start.starts_with("```mermaid") {
                            in_mermaid = true; in_code = true;
                            lines.push(Line::from(Span::styled("  ╭─ mermaid ", Style::default().fg(theme::IRIS()).add_modifier(Modifier::BOLD))));
                        } else if in_mermaid && trimmed_start.starts_with("```") {
                            in_mermaid = false; in_code = false;
                            lines.push(Line::from(Span::styled("  ╰────────── ", Style::default().fg(theme::IRIS()))));
                        } else if in_mermaid {
                            lines.push(Line::from(vec![
                                Span::styled("  │ ", Style::default().fg(theme::IRIS())),
                                Span::styled(raw.trim_end().to_string(), Style::default().fg(theme::GOLD())),
                            ]));
                        } else if trimmed_start.starts_with("```") {
                            if in_code {
                                in_code = false;
                                lines.push(Line::from(Span::styled("  ╰──", Style::default().fg(theme::OVERLAY()))));
                            } else {
                                in_code = true;
                                let lang = trimmed_start.trim_start_matches('`').trim();
                                let label = if lang.is_empty() { "  ╭─ code".to_string() } else { format!("  ╭─ {lang}") };
                                lines.push(Line::from(Span::styled(label, Style::default().fg(theme::OVERLAY()).add_modifier(Modifier::DIM))));
                            }
                        } else {
                            lines.push(render_md_line(raw, in_code));
                        }
                    }
                    let _ = in_code;
                    for tool in &msg.tool_uses {
                        let (icon, col) = match tool.status {
                            ToolUseStatus::Running => (
                                FRAMES[self.spinner_frame % FRAMES.len()].to_string(),
                                theme::GOLD(),
                            ),
                            ToolUseStatus::Completed => ("✓".into(), theme::FOAM()),
                            ToolUseStatus::Error => ("✗".into(), theme::LOVE()),
                        };

                        let header_args = if tool.input_summary.is_empty() {
                            String::new()
                        } else {
                            format!("  {}", tool.input_summary)
                        };
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {icon} "),
                                Style::default().fg(col).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                tool.name.clone(),
                                Style::default().fg(col).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                header_args,
                                Style::default().fg(theme::SUBTLE()),
                            ),
                        ]));

                        if self.tool_details && !tool.diff.is_empty() {
                            for d in &tool.diff {
                                let (sign, sign_fg, body_fg, body_bg) = match d.kind {
                                    DiffLineKind::Added => (
                                        " +",
                                        theme::SUCCESS(),
                                        theme::TEXT(),
                                        Some(theme::HL_MED()),
                                    ),
                                    DiffLineKind::Removed => (
                                        " -",
                                        theme::ERROR(),
                                        theme::TEXT(),
                                        Some(theme::HL_MED()),
                                    ),
                                    DiffLineKind::Context => (
                                        "  ",
                                        theme::TEXT_MUTED(),
                                        theme::TEXT_MUTED(),
                                        None,
                                    ),
                                };
                                let sign_style = Style::default()
                                    .fg(sign_fg)
                                    .add_modifier(Modifier::BOLD);
                                let mut body_style = Style::default().fg(body_fg);
                                if let Some(bg) = body_bg {
                                    body_style = body_style.bg(bg);
                                }
                                lines.push(Line::from(vec![
                                    Span::styled(
                                        "      ┊ ",
                                        Style::default().fg(theme::OVERLAY()),
                                    ),
                                    Span::styled(sign.to_string(), sign_style),
                                    Span::styled(format!(" {}", d.text), body_style),
                                ]));
                            }
                        }

                        let body_lines: &[String] = if !self.tool_details {
                            &[]
                        } else if !tool.diff.is_empty() {
                            &[]
                        } else if tool.output_excerpt.is_empty()
                            && !tool.output_preview.is_empty()
                        {
                            std::slice::from_ref(&tool.output_preview)
                        } else {
                            tool.output_excerpt.as_slice()
                        };

                        for body in body_lines {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "      ┊ ",
                                    Style::default().fg(theme::OVERLAY()),
                                ),
                                Span::styled(
                                    body.clone(),
                                    Style::default()
                                        .fg(theme::MUTED())
                                        .add_modifier(Modifier::DIM),
                                ),
                            ]));
                        }
                    }
                }
            }
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));

        let width = area.width as usize;
        let total_rows: usize = lines
            .iter()
            .map(|l| {
                let w = l.width();
                if w == 0 || width == 0 { 1 } else { (w + width - 1) / width }
            })
            .sum();

        let visible = area.height as usize;
        self.state.total_lines = total_rows;
        let max_offset = total_rows.saturating_sub(visible);
        let offset = if self.state.auto_scroll {
            self.state.scroll_offset = max_offset;
            max_offset
        } else {
            let o = self.state.scroll_offset.min(max_offset);
            self.state.scroll_offset = o;
            o
        };

        Paragraph::new(lines).scroll((offset as u16, 0)).wrap(Wrap { trim: false }).render(area, buf);
    }
}

const LOGO_ART: &[&str] = &[
    r"  ____ _____ __   ___   __ __  ",
    r" / ___|_   _\ \ / / \ | \ \ / / ",
    r" \___ \ | |  \ V /|  \| |\ V /  ",
    r"  ___) || |   | | | |\  | | |   ",
    r" |____/ |_|   |_| |_| \_| |_|   ",
    r"               c o d e          ",
];

fn draw_empty_state(area: Rect, buf: &mut Buffer) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let pad = area.height.saturating_sub(14) / 3;
    for _ in 0..pad { lines.push(Line::from("")); }

    for art in LOGO_ART {
        lines.push(Line::from(Span::styled(
            (*art).to_string(),
            Style::default().fg(theme::PRIMARY()).add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("v{}", env!("CARGO_PKG_VERSION")),
        Style::default().fg(theme::TEXT_MUTED()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let hint = |key: &str, label: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(key.to_string(), Style::default().fg(theme::ACCENT()).add_modifier(Modifier::BOLD)),
            Span::styled("  ".to_string(), Style::default()),
            Span::styled(label.to_string(), Style::default().fg(theme::TEXT_MUTED())),
        ])
    };
    lines.push(hint("^P", "command palette"));
    lines.push(hint("^S", "session list"));
    lines.push(hint("^M", "switch model"));
    lines.push(hint("/help", "show help"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "type a message to begin…".to_string(),
        Style::default().fg(theme::TEXT_MUTED()).add_modifier(Modifier::ITALIC),
    )));

    Paragraph::new(lines).alignment(Alignment::Center).render(area, buf);
}
