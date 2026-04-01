use super::super::terminal::{BOLD, CYAN, DIM, GREEN, ITALIC, RESET, YELLOW};
use super::render_syntax;
use super::RenderState;

pub(super) fn flush_line_buf(state: &mut RenderState) {
    if !state.line_buf.is_empty() {
        let remaining = std::mem::take(&mut state.line_buf);
        render_md_line(&remaining, state);
        println!();
    }
}

pub(super) fn render_md_line(line: &str, state: &mut RenderState) {
    let trimmed = line.trim_end();

    if state.in_code_block {
        if trimmed == "```" || trimmed == "~~~" {
            println!("  {DIM}⎿ ──────────────────────────────────────{RESET}");
            state.in_code_block = false;
            state.code_block_lang = String::new();
            state.highlighter = None;
        } else {
            let rendered = if let Some(ref mut h) = state.highlighter {
                render_syntax::highlight_line(h, trimmed)
            } else {
                format!("{GREEN}{trimmed}{RESET}")
            };
            println!("  {DIM}⎿ │{RESET}  {rendered}");
        }
        return;
    }

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        let fence = if trimmed.starts_with("```") { "```" } else { "~~~" };
        let lang = trimmed.trim_start_matches(fence).trim();
        state.in_code_block = true;
        state.code_block_lang = lang.to_string();
        state.highlighter = render_syntax::new_highlighter(lang);
        let label = if lang.is_empty() {
            String::new()
        } else {
            format!(" {CYAN}{lang}{RESET}{DIM}")
        };
        println!("  {DIM}⎿ ──{label}──────────────────────────────────{RESET}");
        return;
    }

    if matches!(trimmed, "---" | "***" | "___") {
        println!("  {DIM}⎿ ────────────────────────────────────────{RESET}");
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("### ") {
        println!("  {DIM}⎿{RESET} {BOLD}{}{RESET}", render_inline(rest));
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        println!("\n  {DIM}⎿{RESET} {BOLD}{CYAN}{}{RESET}", render_inline(rest));
        return;
    }
    if let Some(rest) = trimmed.strip_prefix("# ") {
        println!("\n  {DIM}⎿{RESET} {BOLD}{CYAN}{}{RESET}", render_inline(rest));
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("> ") {
        println!("  {DIM}⎿ │{RESET} {DIM}{}{RESET}", render_inline(rest));
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        println!("  {DIM}⎿{RESET} {DIM}•{RESET} {}", render_inline(rest));
        return;
    }

    if let Some(pos) = trimmed.find(". ") {
        let prefix = &trimmed[..pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
            let rest = &trimmed[pos + 2..];
            println!("  {DIM}⎿{RESET} {DIM}{}.{RESET} {}", prefix, render_inline(rest));
            return;
        }
    }

    if trimmed.is_empty() {
        println!("  {DIM}⎿{RESET}");
        return;
    }

    println!("  {DIM}⎿{RESET} {}", render_inline(trimmed));
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
