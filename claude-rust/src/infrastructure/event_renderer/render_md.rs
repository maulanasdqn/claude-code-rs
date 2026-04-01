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
        let word_len = word.chars().count();
        let cur_len = current.chars().count();
        if current.is_empty() {
            current.push_str(word);
        } else if cur_len + 1 + word_len <= max {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

pub(super) fn flush_line_buf(state: &mut RenderState) {
    if !state.line_buf.is_empty() {
        let remaining = std::mem::take(&mut state.line_buf);
        render_md_line(&remaining, state);
        println!();
    }
}

pub(super) fn render_md_line(line: &str, state: &mut RenderState) {
    let trimmed = line.trim_end();
    let w = term_width();
    let text_w = w.saturating_sub(4);

    if state.in_code_block {
        if trimmed == "```" || trimmed == "~~~" {
            let bar = "─".repeat(w.saturating_sub(4));
            println!("  {DIM}{bar}{RESET}");
            state.in_code_block = false;
            state.code_block_lang = String::new();
            state.highlighter = None;
        } else {
            let rendered = if let Some(ref mut h) = state.highlighter {
                render_syntax::highlight_line(h, trimmed)
            } else {
                format!("{GREEN}{trimmed}{RESET}")
            };
            println!("  {DIM}│{RESET}  {rendered}");
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
            println!("  {DIM}{bar}{RESET}");
        } else {
            let label = format!(" {CYAN}{lang}{RESET}{DIM} ");
            let fill = w.saturating_sub(4 + lang.len() + 2);
            println!("  {DIM}──{label}{}─{RESET}", "─".repeat(fill));
        }
        return;
    }

    if matches!(trimmed, "---" | "***" | "___") {
        let hr = "─".repeat(text_w);
        println!("  {DIM}{hr}{RESET}");
        return;
    }

    if trimmed.starts_with('|') {
        let inner = trimmed.trim_matches('|');
        let is_sep = inner.split('|').all(|c| c.trim().chars().all(|x| x == '-' || x == ':' || x == ' '));
        if !is_sep {
            println!("  {}", render_inline(trimmed));
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("### ") {
        println!("  {BOLD}{}{RESET}", render_inline(rest));
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        println!("\n  {BOLD}{CYAN}{}{RESET}", render_inline(rest));
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        println!("\n  {BOLD}{CYAN}{}{RESET}", render_inline(rest));
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("> ") {
        for wline in wrap_words(rest, text_w.saturating_sub(4)) {
            println!("  {DIM}│{RESET} {DIM}{}{RESET}", render_inline(&wline));
        }
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        let chunks = wrap_words(rest, text_w.saturating_sub(2));
        println!("  • {}", render_inline(&chunks[0]));
        for chunk in &chunks[1..] {
            println!("    {}", render_inline(chunk));
        }
        return;
    }

    if let Some(pos) = trimmed.find(". ") {
        let prefix = &trimmed[..pos];
        if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
            let rest = &trimmed[pos + 2..];
            let indent_w = prefix.len() + 2;
            let chunks = wrap_words(rest, text_w.saturating_sub(indent_w));
            println!("  {DIM}{prefix}.{RESET} {}", render_inline(&chunks[0]));
            for chunk in &chunks[1..] {
                println!("  {}  {}", " ".repeat(indent_w), render_inline(chunk));
            }
            return;
        }
    }

    if trimmed.is_empty() {
        println!();
        return;
    }

    for wline in wrap_words(trimmed, text_w) {
        println!("  {}", render_inline(&wline));
    }
}

fn render_inline(s: &str) -> String {
    let mut out = String::new();
    let mut rem = s;

    while !rem.is_empty() {
        if rem.starts_with("**") {
            if let Some(end) = rem[2..].find("**") {
                out.push_str(BOLD);
                out.push_str(&rem[2..2 + end]);
                out.push_str(RESET);
                rem = &rem[4 + end..];
                continue;
            }
        }
        if rem.starts_with('*') && !rem.starts_with("**") {
            if let Some(end) = rem[1..].find('*') {
                if end > 0 {
                    out.push_str(ITALIC);
                    out.push_str(&rem[1..1 + end]);
                    out.push_str(RESET);
                    rem = &rem[2 + end..];
                    continue;
                }
            }
        }
        if rem.starts_with('_') && !rem.starts_with("__") {
            if let Some(end) = rem[1..].find('_') {
                if end > 0 {
                    out.push_str(ITALIC);
                    out.push_str(&rem[1..1 + end]);
                    out.push_str(RESET);
                    rem = &rem[2 + end..];
                    continue;
                }
            }
        }
        if rem.starts_with('`') {
            if let Some(end) = rem[1..].find('`') {
                out.push_str(YELLOW);
                out.push_str(&rem[1..1 + end]);
                out.push_str(RESET);
                rem = &rem[2 + end..];
                continue;
            }
        }
        let c = rem.chars().next().expect("non-empty");
        out.push(c);
        rem = &rem[c.len_utf8()..];
    }

    out
}
