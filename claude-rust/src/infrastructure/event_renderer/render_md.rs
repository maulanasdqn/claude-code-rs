use crossterm::terminal;
use super::super::terminal::{BOLD, CYAN, DIM, GREEN, ITALIC, RESET, YELLOW};
use super::render_syntax;
use super::RenderState;

fn term_width() -> usize {
    terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

fn wrap_words(text: &str, max: usize) -> Vec<String> {
    if max < 10 || text.chars().count() <= max {
        return vec![text.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let wlen = word.chars().count();
        let clen = current.chars().count();
        if current.is_empty() {
            current.push_str(word);
        } else if clen + 1 + wlen <= max {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() { lines.push(current); }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

pub(super) fn flush_line_buf(state: &mut RenderState) {
    if !state.line_buf.is_empty() {
        let remaining = std::mem::take(&mut state.line_buf);
        render_md_line(&remaining, state);
    }
    flush_table(state);
    state.mp.println(String::new()).ok();
}

/// Flush buffered table rows as a rendered box-drawing table.
pub(super) fn flush_table(state: &mut RenderState) {
    if state.table_rows.is_empty() { return; }
    let rows = std::mem::take(&mut state.table_rows);
    render_table(&rows, state);
}

pub(super) fn render_md_line(line: &str, state: &mut RenderState) {
    let trimmed = line.trim_end();
    let w = term_width();
    let text_w = w.saturating_sub(4);

    if state.in_code_block {
        if trimmed == "```" || trimmed == "~~~" {
            let bar = "─".repeat(w.saturating_sub(4));
            state.mp.println(format!("  {DIM}{bar}{RESET}")).ok();
            state.in_code_block = false;
            state.code_block_lang = String::new();
            state.highlighter = None;
        } else {
            let rendered = if let Some(ref mut h) = state.highlighter {
                render_syntax::highlight_line(h, trimmed)
            } else {
                format!("{GREEN}{trimmed}{RESET}")
            };
            state.mp.println(format!("  {DIM}│{RESET}  {rendered}")).ok();
        }
        return;
    }

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        let fence = if trimmed.starts_with("```") { "```" } else { "~~~" };
        let lang = trimmed.trim_start_matches(fence).trim();
        state.in_code_block = true;
        state.code_block_lang = lang.to_string();
        state.highlighter = render_syntax::new_highlighter(lang);
        let bar = "─".repeat(w.saturating_sub(4));
        if lang.is_empty() {
            state.mp.println(format!("  {DIM}{bar}{RESET}")).ok();
        } else {
            let fill = w.saturating_sub(6 + lang.len());
            state.mp.println(format!("  {DIM}── {CYAN}{lang}{RESET}{DIM} {}─{RESET}", "─".repeat(fill))).ok();
        }
        return;
    }

    if matches!(trimmed, "---" | "***" | "___") {
        let hr = "─".repeat(text_w);
        state.mp.println(format!("  {DIM}{hr}{RESET}")).ok();
        return;
    }

    if trimmed.starts_with('|') {
        let inner = trimmed.trim_matches('|');
        let is_sep = inner.split('|').all(|c| c.trim().chars().all(|x| x == '-' || x == ':' || x == ' '));
        if !is_sep {
            // Parse cells and buffer; render when the table ends
            let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_string()).collect();
            state.table_rows.push(cells);
        }
        return;
    }

    // Non-table line — flush any buffered table first
    if !state.table_rows.is_empty() {
        flush_table(state);
    }

    if let Some(rest) = trimmed.strip_prefix("### ") {
        state.mp.println(format!("  {BOLD}{}{RESET}", render_inline(rest))).ok();
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        state.mp.println(String::new()).ok();
        state.mp.println(format!("  {BOLD}{CYAN}{}{RESET}", render_inline(rest))).ok();
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        state.mp.println(String::new()).ok();
        state.mp.println(format!("  {BOLD}{CYAN}{}{RESET}", render_inline(rest))).ok();
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("> ") {
        for wline in wrap_words(rest, text_w.saturating_sub(4)) {
            state.mp.println(format!("  {DIM}│{RESET} {DIM}{}{RESET}", render_inline(&wline))).ok();
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        let chunks = wrap_words(rest, text_w.saturating_sub(2));
        state.mp.println(format!("  • {}", render_inline(&chunks[0]))).ok();
        for chunk in &chunks[1..] {
            state.mp.println(format!("    {}", render_inline(chunk))).ok();
        }
        return;
    }

    if let Some(pos) = trimmed.find(". ") {
        let prefix = &trimmed[..pos];
        if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
            let rest = &trimmed[pos + 2..];
            let indent_w = prefix.len() + 2;
            let chunks = wrap_words(rest, text_w.saturating_sub(indent_w));
            state.mp.println(format!("  {DIM}{prefix}.{RESET} {}", render_inline(&chunks[0]))).ok();
            for chunk in &chunks[1..] {
                state.mp.println(format!("  {}  {}", " ".repeat(indent_w), render_inline(chunk))).ok();
            }
            return;
        }
    }

    if trimmed.is_empty() {
        state.mp.println(String::new()).ok();
        return;
    }

    for wline in wrap_words(trimmed, text_w) {
        state.mp.println(format!("  {}", render_inline(&wline))).ok();
    }
}

/// Render a buffered table as an aligned box-drawing table.
fn render_table(rows: &[Vec<String>], state: &mut RenderState) {
    if rows.is_empty() { return; }

    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 { return; }

    // Render each cell with inline formatting; measure visible width separately.
    let rendered: Vec<Vec<String>> = rows.iter().map(|row| {
        (0..ncols).map(|i| {
            row.get(i).map(|c| render_inline(c)).unwrap_or_default()
        }).collect()
    }).collect();

    let col_widths: Vec<usize> = (0..ncols).map(|c| {
        rendered.iter()
            .map(|row| visible_width(&row[c]))
            .max()
            .unwrap_or(0)
            .max(3)
    }).collect();

    let top_border: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("┌{}", "─".repeat(w + 2)) } else { format!("┬{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┐";

    let mid_border: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("├{}", "─".repeat(w + 2)) } else { format!("┼{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┤";

    let bot_border: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("└{}", "─".repeat(w + 2)) } else { format!("┴{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┘";

    state.mp.println(format!("  {DIM}{top_border}{RESET}")).ok();

    for (row_i, row) in rendered.iter().enumerate() {
        let cells: String = row.iter().enumerate().map(|(c, cell)| {
            let vw = visible_width(cell);
            let pad = " ".repeat(col_widths[c].saturating_sub(vw));
            format!("{DIM}│{RESET} {cell}{pad} ")
        }).collect::<String>() + &format!("{DIM}│{RESET}");
        state.mp.println(format!("  {cells}")).ok();

        // Separator line after header row
        if row_i == 0 && rows.len() > 1 {
            state.mp.println(format!("  {DIM}{mid_border}{RESET}")).ok();
        }
    }

    state.mp.println(format!("  {DIM}{bot_border}{RESET}")).ok();
    state.mp.println(String::new()).ok();
}

/// Measure visible terminal width of a string, stripping ANSI escape codes.
/// Wide characters (CJK, emoji) count as 2 columns.
fn visible_width(s: &str) -> usize {
    let mut width = 0usize;
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' { in_esc = true; continue; }
        if in_esc { if c == 'm' { in_esc = false; } continue; }
        // Emoji and wide unicode characters occupy 2 columns
        width += if (c as u32) > 0x2E7F { 2 } else { 1 };
    }
    width
}

fn render_inline(s: &str) -> String {
    let mut out = String::new();
    let mut rem = s;

    while !rem.is_empty() {
        if rem.starts_with("**")
            && let Some(end) = rem[2..].find("**") {
                out.push_str(BOLD);
                out.push_str(&rem[2..2 + end]);
                out.push_str(RESET);
                rem = &rem[4 + end..];
                continue;
            }
        if rem.starts_with('*') && !rem.starts_with("**")
            && let Some(end) = rem[1..].find('*')
                && end > 0 {
                    out.push_str(ITALIC);
                    out.push_str(&rem[1..1 + end]);
                    out.push_str(RESET);
                    rem = &rem[2 + end..];
                    continue;
                }
        if rem.starts_with('_') && !rem.starts_with("__")
            && let Some(end) = rem[1..].find('_')
                && end > 0 {
                    out.push_str(ITALIC);
                    out.push_str(&rem[1..1 + end]);
                    out.push_str(RESET);
                    rem = &rem[2 + end..];
                    continue;
                }
        if rem.starts_with('`')
            && let Some(end) = rem[1..].find('`') {
                out.push_str(YELLOW);
                out.push_str(&rem[1..1 + end]);
                out.push_str(RESET);
                rem = &rem[2 + end..];
                continue;
            }
        let c = rem.chars().next().expect("non-empty");
        out.push(c);
        rem = &rem[c.len_utf8()..];
    }

    out
}
