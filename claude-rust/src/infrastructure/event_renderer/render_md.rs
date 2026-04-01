use super::super::terminal::{BOLD, CYAN, DIM, ITALIC, RESET};
use super::RenderState;

pub(super) fn flush_line_buf(state: &mut RenderState) {
    if !state.line_buf.is_empty() {
        let remaining = std::mem::take(&mut state.line_buf);
        render_md_line(&remaining);
        println!();
    }
}

pub(super) fn render_md_line(line: &str) {
    let trimmed = line.trim_end();

    if matches!(trimmed, "---" | "***" | "___") {
        println!("  {DIM}────────────────────────────────────────{RESET}");
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
        println!("  {DIM}│{RESET} {DIM}{}{RESET}", render_inline(rest));
        return;
    }

    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        println!("  {DIM}•{RESET} {}", render_inline(rest));
        return;
    }

    if let Some(pos) = trimmed.find(". ") {
        let prefix = &trimmed[..pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
            let rest = &trimmed[pos + 2..];
            println!("  {DIM}{}.{RESET} {}", prefix, render_inline(rest));
            return;
        }
    }

    if trimmed.is_empty() {
        println!();
        return;
    }

    println!("  {}", render_inline(trimmed));
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
                out.push_str(CYAN);
                out.push_str(&rem[1..1 + end]);
                out.push_str(RESET);
                rem = &rem[2 + end..];
                continue;
            }
        }
        let c = rem.chars().next().expect("non-empty rem must have a char");
        out.push(c);
        rem = &rem[c.len_utf8()..];
    }

    out
}
