use crossterm::terminal;
use super::super::terminal::{BOLD, CYAN, DIM, GREEN, RESET};
use super::render_syntax;
use super::render_table::{render_inline, render_table, wrap_words};
use super::RenderState;

fn term_width() -> usize {
    terminal::size().map(|(w, _)| w as usize).unwrap_or(80)
}

pub(super) fn flush_line_buf(state: &mut RenderState) {
    if !state.line_buf.is_empty() {
        let remaining = std::mem::take(&mut state.line_buf);
        render_md_line(&remaining, state);
    }
    flush_table(state);
    state.mp.println(String::new()).ok();
}

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
            let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_string()).collect();
            state.table_rows.push(cells);
        }
        return;
    }
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
