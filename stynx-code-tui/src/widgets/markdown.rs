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

/// True for any pipe-delimited markdown table line (data row or separator).
pub(super) fn is_table_line(s: &str) -> bool {
    is_table_row(s)
}

/// Rendered display width of a cell (markdown stripped, e.g. `` `code` `` → `code`).
fn cell_text_width(cell: &str) -> usize {
    parse_inline(cell).iter().map(|s| s.content.chars().count()).sum()
}

/// Greedy word-wrap a run of styled spans into lines no wider than `width`.
/// Words longer than the column (e.g. long file paths) are hard-split so they
/// never overflow. Returns at least one (possibly empty) line.
fn wrap_spans(spans: &[Span<'static>], width: usize) -> Vec<Vec<Span<'static>>> {
    let width = width.max(1);
    let mut words: Vec<(String, Style)> = Vec::new();
    for s in spans {
        for w in s.content.split(' ') {
            if !w.is_empty() {
                words.push((w.to_string(), s.style));
            }
        }
    }

    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut cur: Vec<Span<'static>> = Vec::new();
    let mut cur_w = 0usize;

    for (word, style) in words {
        let wlen = word.chars().count();
        if wlen > width {
            // Hard-split an unbreakable token across as many lines as needed.
            if cur_w > 0 {
                lines.push(std::mem::take(&mut cur));
                cur_w = 0;
            }
            let chars: Vec<char> = word.chars().collect();
            let mut idx = 0;
            while idx < chars.len() {
                let take = (width - cur_w).min(chars.len() - idx).max(1);
                let chunk: String = chars[idx..idx + take].iter().collect();
                cur.push(Span::styled(chunk, style));
                cur_w += take;
                idx += take;
                if idx < chars.len() {
                    lines.push(std::mem::take(&mut cur));
                    cur_w = 0;
                }
            }
            continue;
        }
        let need = if cur_w == 0 { wlen } else { wlen + 1 };
        if cur_w > 0 && cur_w + need > width {
            lines.push(std::mem::take(&mut cur));
            cur_w = 0;
        }
        if cur_w > 0 {
            cur.push(Span::styled(" ".to_string(), Style::default()));
            cur_w += 1;
        }
        cur.push(Span::styled(word, style));
        cur_w += wlen;
    }
    lines.push(cur);
    lines
}

/// Render a whole table block at once. Columns are measured across all rows and
/// constrained to the viewport `avail_width`; cells that don't fit wrap *within
/// their own column* (continuations stay under the column, never overflowing to
/// the left edge), and each row grows to the tallest wrapped cell.
pub(super) fn render_table_block(rows: &[&str], avail_width: usize) -> Vec<Line<'static>> {
    let parsed: Vec<(bool, Vec<String>)> = rows
        .iter()
        .map(|r| {
            let sep = is_table_sep(r);
            let cells = r
                .trim()
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect::<Vec<_>>();
            (sep, cells)
        })
        .collect();

    let ncols = parsed.iter().map(|(_, c)| c.len()).max().unwrap_or(0);
    if ncols == 0 {
        return Vec::new();
    }

    // Natural (unconstrained) column widths from the rendered cell text.
    let mut widths = vec![0usize; ncols];
    for (sep, cells) in &parsed {
        if *sep {
            continue;
        }
        for (i, c) in cells.iter().enumerate() {
            widths[i] = widths[i].max(cell_text_width(c));
        }
    }

    // Fit within the viewport: 2-col left margin + " │ " (3 cols) between
    // columns, with a little safety margin so the outer paragraph never re-wraps.
    let margin = 2usize;
    let sep_total = 3 * ncols.saturating_sub(1);
    let budget = avail_width
        .saturating_sub(margin + sep_total + 1)
        .max(ncols * 4);
    let min_col = 4usize;
    let mut guard = 0;
    while widths.iter().sum::<usize>() > budget && guard < 100_000 {
        let widest = widths
            .iter()
            .enumerate()
            .filter(|(_, w)| **w > min_col)
            .max_by_key(|(_, w)| **w)
            .map(|(i, _)| i);
        match widest {
            Some(i) => widths[i] -= 1,
            None => break,
        }
        guard += 1;
    }
    let table_w: usize = widths.iter().sum::<usize>() + sep_total;

    let mut out: Vec<Line<'static>> = Vec::new();
    let mut header_done = false;
    for (sep, cells) in &parsed {
        if *sep {
            out.push(Line::from(Span::styled(
                format!("  {}", "─".repeat(table_w)),
                Style::default().fg(theme::OVERLAY()).add_modifier(Modifier::DIM),
            )));
            continue;
        }
        let is_header = !header_done;
        header_done = true;

        // Wrap every cell to its column width up front.
        let wrapped: Vec<Vec<Vec<Span<'static>>>> = (0..ncols)
            .map(|i| {
                let empty = String::new();
                let cell = cells.get(i).unwrap_or(&empty);
                let mut spans = parse_inline(cell);
                if is_header {
                    for s in &mut spans {
                        s.style = s.style.fg(theme::ROSE()).add_modifier(Modifier::BOLD);
                    }
                }
                wrap_spans(&spans, widths[i])
            })
            .collect();

        let row_h = wrapped.iter().map(|c| c.len()).max().unwrap_or(1).max(1);
        for k in 0..row_h {
            let mut spans: Vec<Span<'static>> = vec![Span::styled("  ".to_string(), Style::default())];
            for i in 0..ncols {
                if i > 0 {
                    spans.push(Span::styled(" │ ", Style::default().fg(theme::OVERLAY())));
                }
                let seg = wrapped[i].get(k);
                let seg_w: usize = seg
                    .map(|ss| ss.iter().map(|s| s.content.chars().count()).sum())
                    .unwrap_or(0);
                if let Some(ss) = seg {
                    spans.extend(ss.iter().cloned());
                }
                if widths[i] > seg_w {
                    spans.push(Span::styled(" ".repeat(widths[i] - seg_w), Style::default()));
                }
            }
            out.push(Line::from(spans));
        }
    }
    out
}

/// Assemble wrapped body segments into lines with a hanging indent: the first
/// line carries `prefix`; continuation lines are indented to the prefix width so
/// wrapped text aligns under the content instead of overflowing to the left.
fn assemble_wrapped(
    prefix: &str,
    prefix_style: Style,
    body_lines: Vec<Vec<Span<'static>>>,
    hang: usize,
) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    for (k, segs) in body_lines.into_iter().enumerate() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        if k == 0 {
            spans.push(Span::styled(prefix.to_string(), prefix_style));
        } else {
            spans.push(Span::styled(" ".repeat(hang), Style::default()));
        }
        spans.extend(segs);
        out.push(Line::from(spans));
    }
    if out.is_empty() {
        out.push(Line::from(Span::styled(prefix.to_string(), prefix_style)));
    }
    out
}

/// Width-aware markdown line: wraps the content to `width` with a hanging indent
/// so continuation lines align under the bullet/heading/paragraph text.
pub(super) fn render_md_line_wrapped(raw: &str, in_code: bool, width: usize) -> Vec<Line<'static>> {
    let trimmed = raw.trim_end();
    let width = width.max(8);

    // Code, horizontal rules and table rows keep their existing single-line form
    // (tables are intercepted as a block by the caller before reaching here).
    if in_code || is_hr(trimmed) || is_table_line(trimmed) {
        return vec![render_md_line(trimmed, in_code)];
    }

    // Headings: whole line styled, indented 2.
    let heading = if let Some(r) = trimmed.strip_prefix("### ") {
        Some((r, Style::default().fg(theme::FOAM()).add_modifier(Modifier::BOLD)))
    } else if let Some(r) = trimmed.strip_prefix("## ") {
        Some((r, Style::default().fg(theme::ROSE()).add_modifier(Modifier::BOLD)))
    } else if let Some(r) = trimmed.strip_prefix("# ") {
        Some((r, Style::default().fg(theme::GOLD()).add_modifier(Modifier::BOLD)))
    } else {
        None
    };
    if let Some((text, style)) = heading {
        let hang = 2;
        let segs = wrap_spans(&[Span::styled(text.to_string(), style)], width.saturating_sub(hang).max(1));
        return assemble_wrapped("  ", Style::default(), segs, hang);
    }

    // Bullets (nested) and plain paragraphs.
    let (prefix, body) = if let Some(r) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        ("  • ", r)
    } else if let Some(r) = trimmed.strip_prefix("  - ").or_else(|| trimmed.strip_prefix("  * ")) {
        ("    ◦ ", r)
    } else if let Some(r) = trimmed.strip_prefix("    - ").or_else(|| trimmed.strip_prefix("    * ")) {
        ("      · ", r)
    } else {
        ("  ", trimmed)
    };
    let hang = prefix.chars().count();
    let body_spans = parse_inline(body);
    let segs = wrap_spans(&body_spans, width.saturating_sub(hang).max(1));
    assemble_wrapped(prefix, Style::default().fg(theme::PINE()), segs, hang)
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
