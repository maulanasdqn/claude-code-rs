use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::state::{AppState, ToolUseStatus};
use crate::theme;
use crate::widgets::spinner::FRAMES;

pub struct ToolHistory<'a> {
    pub state: &'a mut AppState,
}

impl<'a> ToolHistory<'a> {
    pub fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }
}

#[derive(Clone, Copy)]
pub enum HistoryRow {
    Tool { msg: usize, tool: usize },
    Sub { msg: usize, tool: usize, sub: usize },
}

pub fn flat_tools(state: &AppState) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (mi, m) in state.conversation.messages.iter().enumerate() {
        for (ti, _t) in m.tool_uses.iter().enumerate() {
            out.push((mi, ti));
        }
    }
    out
}

pub fn flat_rows(state: &AppState) -> Vec<HistoryRow> {
    let mut out = Vec::new();
    for (mi, m) in state.conversation.messages.iter().enumerate() {
        for (ti, t) in m.tool_uses.iter().enumerate() {
            out.push(HistoryRow::Tool { msg: mi, tool: ti });
            for (si, _) in t.sub_progress.iter().enumerate() {
                out.push(HistoryRow::Sub { msg: mi, tool: ti, sub: si });
            }
        }
    }
    out
}

fn pretty_name(name: &str) -> &str {
    match name {
        "bash" => "Bash",
        "read" => "Read",
        "file_write" => "Write",
        "file_edit" => "Edit",
        "glob" => "Glob",
        "grep" => "Grep",
        "web_fetch" => "WebFetch",
        "web_search" => "WebSearch",
        "todo_write" => "TodoWrite",
        "todo_read" => "TodoRead",
        "ask_user_question" => "AskUser",
        "agent" => "Agent",
        "explore" => "Explore",
        "delegate_to_all_interns" => "AllInterns",
        other => other,
    }
}

fn truncate_path(s: &str, max: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let total: usize = s.chars().map(|c| c.width().unwrap_or(0)).sum();
    if total <= max { return s.to_string(); }
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if let Some(last) = parts.last() {
            let last_w: usize = last.chars().map(|c| c.width().unwrap_or(0)).sum();
            if last_w + 4 <= max {
                return format!("…/{last}");
            }
        }
    }
    let mut out = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw + 1 > max { break; }
        w += cw;
        out.push(c);
    }
    out.push('…');
    out
}

impl<'a> Widget for ToolHistory<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(theme::OVERLAY()));
        let inner = block.inner(area);
        block.render(area, buf);

        let header = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "Tools",
                Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if self.state.tool_history.focused { "  ●" } else { "" },
                Style::default().fg(theme::IRIS()),
            ),
        ]);

        let mut lines: Vec<Line<'static>> = vec![header, Line::from("")];

        let rows = flat_rows(self.state);
        let row_width = inner.width.saturating_sub(2) as usize;
        let name_w = 10usize;
        let summary_w = row_width.saturating_sub(name_w + 4);
        let sub_w = row_width.saturating_sub(4);

        let visible = (inner.height as usize).saturating_sub(2).max(1);
        let total = rows.len();
        let selected = self.state.tool_history.selected;
        if let Some(sel) = selected {
            let scr = &mut self.state.tool_history.scroll;
            if sel < *scr { *scr = sel; }
            else if sel >= *scr + visible { *scr = sel + 1 - visible; }
        } else if total > visible {
            self.state.tool_history.scroll = total - visible;
        }
        let scroll = self.state.tool_history.scroll.min(total.saturating_sub(visible));

        for (i, row) in rows.iter().enumerate().skip(scroll).take(visible) {
            let is_selected = selected == Some(i);
            let row_style = if is_selected {
                Style::default().bg(theme::HL_MED()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_selected { "▶ " } else { "  " };

            match row {
                HistoryRow::Tool { msg, tool } => {
                    let tool = &self.state.conversation.messages[*msg].tool_uses[*tool];
                    let (dot, dot_col) = match tool.status {
                        ToolUseStatus::Running => (
                            FRAMES[self.state.spinner_frame % FRAMES.len()].to_string(),
                            theme::GOLD(),
                        ),
                        ToolUseStatus::Completed => ("●".into(), theme::SUCCESS()),
                        ToolUseStatus::Error => ("●".into(), theme::ERROR()),
                    };
                    let pretty = pretty_name(&tool.name);
                    let name_padded = format!("{:<name_w$}", pretty, name_w = name_w);
                    // While a tool is running and producing output, stream the
                    // latest output line in place of the static input summary.
                    let summary = if tool.status == ToolUseStatus::Running
                        && !tool.output_preview.trim().is_empty()
                    {
                        truncate_path(tool.output_preview.trim(), summary_w)
                    } else {
                        truncate_path(&tool.input_summary, summary_w)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(theme::IRIS()).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{dot} "), Style::default().fg(dot_col).add_modifier(Modifier::BOLD)),
                        Span::styled(name_padded, row_style.fg(theme::TEXT())),
                        Span::styled(summary, row_style.fg(theme::TEXT_MUTED())),
                    ]));
                }
                HistoryRow::Sub { msg, tool, sub } => {
                    let parent = &self.state.conversation.messages[*msg].tool_uses[*tool];
                    let text = parent.sub_progress.get(*sub).cloned().unwrap_or_default();
                    let truncated = truncate_path(&text, sub_w);
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(theme::IRIS())),
                        Span::styled("↪ ", Style::default().fg(theme::IRIS()).add_modifier(Modifier::DIM)),
                        Span::styled(truncated, row_style.fg(theme::SUBTLE()).add_modifier(Modifier::ITALIC)),
                    ]));
                }
            }
        }

        if total == 0 {
            lines.push(Line::from(Span::styled(
                "  no tool calls yet",
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::ITALIC),
            )));
        } else if total > visible {
            lines.push(Line::from(Span::styled(
                format!("  {} / {} rows", (scroll + visible).min(total), total),
                Style::default().fg(theme::SUBTLE()).add_modifier(Modifier::DIM),
            )));
        }

        Paragraph::new(lines).render(inner, buf);
    }
}
