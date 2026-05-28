use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use ratatui::layout::Alignment;

use crate::state::ConversationState;
use crate::theme;
use super::markdown::render_md_line;

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

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pad = if area.width >= 80 { 4 } else { 2 };
        let area = Rect {
            x: area.x + pad,
            width: area.width.saturating_sub(pad * 2),
            y: area.y + 1,
            height: area.height.saturating_sub(1),
        };

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
                "done" => {
                    let parts: Vec<&str> = msg.content.split(", ").collect();
                    for (i, part) in parts.iter().enumerate() {
                        let prefix = if i == 0 { "  ✓ " } else { "    " };
                        let prefix_style = if i == 0 {
                            Style::default().fg(theme::SUCCESS()).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        lines.push(Line::from(vec![
                            Span::styled(prefix, prefix_style),
                            Span::styled(
                                part.to_string(),
                                Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }
                }
                _ => {
                    if !msg.thinking.is_empty() && !msg.is_streaming {
                        let lc = msg.thinking.lines().count();
                        lines.push(Line::from(Span::styled(
                            format!("  ◈ thinking · {lc} lines"),
                            Style::default()
                                .fg(theme::MUTED())
                                .add_modifier(Modifier::ITALIC | Modifier::DIM),
                        )));
                        lines.push(Line::from(""));
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
    r" ____   _____ __   __ _   _ __  __",
    r"/ ___| |_   _|\ \ / /| \ | |\ \/ /",
    r"\___ \   | |   \ V / |  \| | \  / ",
    r" ___) |  | |    | |  | |\  | /  \ ",
    r"|____/   |_|    |_|  |_| \_|/_/\_\",
];
const LOGO_SUBTITLE: &str = "c   o   d   e";

const HINTS: &[(&str, &str)] = &[
    ("^P",    "command palette"),
    ("^T",    "focus tools"),
    ("^M",    "switch model"),
    ("/help", "show help"),
];

fn draw_empty_state(area: Rect, buf: &mut Buffer) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let logo_w = LOGO_ART.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let pad_to = |s: &str, w: usize| -> String {
        let n = s.chars().count();
        if n >= w { s.to_string() } else {
            let extra = w - n;
            let left = extra / 2;
            let right = extra - left;
            format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
        }
    };

    let top_pad = area.height.saturating_sub(16) / 3;
    for _ in 0..top_pad { lines.push(Line::from("")); }

    for art in LOGO_ART {
        lines.push(Line::from(Span::styled(
            pad_to(art, logo_w),
            Style::default().fg(theme::PRIMARY()).add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(Line::from(Span::styled(
        pad_to(LOGO_SUBTITLE, logo_w),
        Style::default().fg(theme::ACCENT()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("v{}", env!("CARGO_PKG_VERSION")),
        Style::default().fg(theme::TEXT_MUTED()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "────────────────────".to_string(),
        Style::default().fg(theme::BORDER()),
    )));
    lines.push(Line::from(""));

    let key_w = HINTS.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
    let label_w = HINTS.iter().map(|(_, l)| l.chars().count()).max().unwrap_or(0);
    for (k, l) in HINTS {
        let key = format!("{:>width$}", k, width = key_w);
        let label = format!("{:<width$}", l, width = label_w);
        lines.push(Line::from(vec![
            Span::styled(key, Style::default().fg(theme::ACCENT()).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(label, Style::default().fg(theme::TEXT_MUTED())),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "type a message to begin…".to_string(),
        Style::default().fg(theme::TEXT_MUTED()).add_modifier(Modifier::ITALIC),
    )));

    Paragraph::new(lines).alignment(Alignment::Center).render(area, buf);
}
