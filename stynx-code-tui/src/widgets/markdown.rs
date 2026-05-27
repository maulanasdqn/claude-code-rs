use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme;

pub(super) fn parse_inline(text: &str) -> Vec<Span<'static>> {
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

pub(super) fn render_md_line(raw: &str, in_code: bool) -> Line<'static> {
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
