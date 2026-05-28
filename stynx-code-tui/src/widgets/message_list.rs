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
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  ✓ ",
                            Style::default().fg(theme::SUCCESS()).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            msg.content.clone(),
                            Style::default().fg(theme::MUTED()).add_modifier(Modifier::ITALIC),
                        ),
                    ]));
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
                    let has_content = !msg.content.trim().is_empty();
                    let has_tools = !msg.tool_uses.is_empty();
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
                    if has_content && has_tools {
                        lines.push(Line::from(""));
                    }
                    for tool in &msg.tool_uses {
                        let (dot, col) = match tool.status {
                            ToolUseStatus::Running => (
                                FRAMES[self.spinner_frame % FRAMES.len()].to_string(),
                                theme::GOLD(),
                            ),
                            ToolUseStatus::Completed => ("●".into(), theme::SUCCESS()),
                            ToolUseStatus::Error => ("●".into(), theme::ERROR()),
                        };

                        let pretty_name = pretty_tool_name(&tool.name);
                        let header_text = if tool.input_summary.is_empty() {
                            pretty_name.to_string()
                        } else {
                            format!("{pretty_name}({})", tool.input_summary)
                        };
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {dot} "),
                                Style::default().fg(col).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                header_text,
                                Style::default().fg(theme::TEXT()).add_modifier(Modifier::BOLD),
                            ),
                        ]));

                        if self.tool_details && !tool.diff.is_empty() {

                            let max_diff_show = 6usize;
                            let total_diff = tool.diff.len();
                            let show = total_diff.min(max_diff_show);
                            for (i, d) in tool.diff.iter().take(show).enumerate() {
                                let (sign, sign_fg, body_fg, body_bg) = match d.kind {
                                    DiffLineKind::Added => (
                                        "+",
                                        theme::SUCCESS(),
                                        theme::TEXT(),
                                        Some(theme::HL_MED()),
                                    ),
                                    DiffLineKind::Removed => (
                                        "-",
                                        theme::ERROR(),
                                        theme::TEXT(),
                                        Some(theme::HL_MED()),
                                    ),
                                    DiffLineKind::Context => (
                                        " ",
                                        theme::TEXT_MUTED(),
                                        theme::TEXT_MUTED(),
                                        None,
                                    ),
                                };
                                let prefix = if i == 0 { "  ⎿  " } else { "     " };
                                let sign_style = Style::default()
                                    .fg(sign_fg)
                                    .add_modifier(Modifier::BOLD);
                                let mut body_style = Style::default().fg(body_fg);
                                if let Some(bg) = body_bg {
                                    body_style = body_style.bg(bg);
                                }
                                lines.push(Line::from(vec![
                                    Span::styled(
                                        prefix,
                                        Style::default().fg(theme::OVERLAY()),
                                    ),
                                    Span::styled(sign.to_string(), sign_style),
                                    Span::styled(format!(" {}", d.text), body_style),
                                ]));
                            }
                            if total_diff > show {
                                lines.push(Line::from(vec![
                                    Span::styled(
                                        "     ",
                                        Style::default().fg(theme::OVERLAY()),
                                    ),
                                    Span::styled(
                                        format!("… +{} lines (ctrl+o to expand)", total_diff - show),
                                        Style::default()
                                            .fg(theme::TEXT_MUTED())
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ]));
                            }
                        }

                        let suppress_body = matches!(tool.name.as_str(), "read");
                        let body_lines: &[String] = if !self.tool_details {
                            &[]
                        } else if !tool.diff.is_empty() || suppress_body {
                            &[]
                        } else if tool.output_excerpt.is_empty()
                            && !tool.output_preview.is_empty()
                        {
                            std::slice::from_ref(&tool.output_preview)
                        } else {
                            tool.output_excerpt.as_slice()
                        };

                        let max_body_show = 2usize;
                        let total_body = body_lines.len();
                        let show = total_body.min(max_body_show);
                        let body_width = (area.width as usize).saturating_sub(7).max(20);
                        for (i, body) in body_lines.iter().take(show).enumerate() {
                            let cleaned = clean_tool_body_line(&tool.name, body);
                            let truncated = truncate_to_width(&cleaned, body_width);
                            let prefix = if i == 0 { "  ⎿  " } else { "     " };
                            lines.push(Line::from(vec![
                                Span::styled(
                                    prefix,
                                    Style::default().fg(theme::OVERLAY()),
                                ),
                                Span::styled(
                                    truncated,
                                    Style::default().fg(theme::TEXT_MUTED()),
                                ),
                            ]));
                        }
                        if total_body > show {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "     ",
                                    Style::default().fg(theme::OVERLAY()),
                                ),
                                Span::styled(
                                    format!("… +{} lines (ctrl+o to expand)", total_body - show),
                                    Style::default()
                                        .fg(theme::TEXT_MUTED())
                                        .add_modifier(Modifier::ITALIC),
                                ),
                            ]));
                        }

                        if !tool.sub_progress.is_empty() && self.tool_details {
                            let progress_w = (area.width as usize).saturating_sub(9).max(20);
                            let recent = tool.sub_progress.iter().rev().take(8).collect::<Vec<_>>();
                            for (i, line) in recent.iter().rev().enumerate() {
                                let truncated = truncate_to_width(line, progress_w);
                                let prefix = if i == 0 { "     ↪ " } else { "       " };
                                lines.push(Line::from(vec![
                                    Span::styled(prefix, Style::default().fg(theme::IRIS())),
                                    Span::styled(truncated, Style::default().fg(theme::SUBTLE())),
                                ]));
                            }
                            if tool.sub_progress.len() > recent.len() {
                                lines.push(Line::from(vec![
                                    Span::styled("       ", Style::default()),
                                    Span::styled(
                                        format!("… +{} earlier steps", tool.sub_progress.len() - recent.len()),
                                        Style::default().fg(theme::TEXT_MUTED()).add_modifier(Modifier::ITALIC),
                                    ),
                                ]));
                            }
                        }

                        lines.push(Line::from(""));
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

fn truncate_to_width(s: &str, max: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut out = String::with_capacity(s.len());
    let mut width = 0usize;
    let mut truncated = false;
    for c in s.chars() {
        let w = c.width().unwrap_or(0);
        if width + w > max.saturating_sub(1) {
            truncated = true;
            break;
        }
        width += w;
        out.push(c);
    }
    if truncated {
        out.push('…');
    }
    out
}

fn pretty_tool_name(name: &str) -> String {
    match name {
        "bash" => "Bash".into(),
        "read" => "Read".into(),
        "file_write" => "Write".into(),
        "file_edit" => "Edit".into(),
        "glob" => "Glob".into(),
        "grep" => "Grep".into(),
        "web_fetch" => "WebFetch".into(),
        "web_search" => "WebSearch".into(),
        "todo_write" => "TodoWrite".into(),
        "todo_read" => "TodoRead".into(),
        "ask_user_question" => "AskUser".into(),
        "agent" => "Agent".into(),
        "explore" => "Explore".into(),
        _ => {

            name.split('_')
                .map(|seg| {
                    let mut chars = seg.chars();
                    match chars.next() {
                        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect()
        }
    }
}

fn clean_tool_body_line(tool: &str, raw: &str) -> String {
    let line = raw.trim_end();
    if tool == "read" {

        if let Some(rest) = strip_read_prefix(line) {
            return rest.to_string();
        }
    }
    line.to_string()
}

fn strip_read_prefix(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i] == b' ' { i += 1; }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i == digits_start { return None; }
    if i >= bytes.len() { return None; }
    if bytes[i] == b'\t' { return Some(&s[i + 1..]); }
    if bytes[i] == b' ' {
        let mut j = i;
        while j < bytes.len() && bytes[j] == b' ' { j += 1; }
        if j - i >= 2 {
            return Some(&s[j..]);
        }
    }
    None
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
    ("^S",    "session list"),
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
