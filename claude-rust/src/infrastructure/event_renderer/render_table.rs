use super::super::terminal::{BOLD, DIM, ITALIC, RESET, YELLOW};
use super::RenderState;

pub(super) fn wrap_words(text: &str, max: usize) -> Vec<String> {
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

pub(super) fn visible_width(s: &str) -> usize {
    let mut width = 0usize;
    let mut in_esc = false;
    for c in s.chars() {
        if c == '\x1b' { in_esc = true; continue; }
        if in_esc { if c == 'm' { in_esc = false; } continue; }
        width += if (c as u32) > 0x2E7F { 2 } else { 1 };
    }
    width
}

pub(super) fn render_inline(s: &str) -> String {
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

pub(super) fn render_table(rows: &[Vec<String>], state: &mut RenderState) {
    if rows.is_empty() { return; }
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 { return; }
    let rendered: Vec<Vec<String>> = rows.iter().map(|row| {
        (0..ncols).map(|i| row.get(i).map(|c| render_inline(c)).unwrap_or_default()).collect()
    }).collect();
    let col_widths: Vec<usize> = (0..ncols).map(|c| {
        rendered.iter().map(|row| visible_width(&row[c])).max().unwrap_or(0).max(3)
    }).collect();
    let top: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("┌{}", "─".repeat(w + 2)) } else { format!("┬{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┐";
    let mid: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("├{}", "─".repeat(w + 2)) } else { format!("┼{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┤";
    let bot: String = col_widths.iter().enumerate()
        .map(|(i, &w)| if i == 0 { format!("└{}", "─".repeat(w + 2)) } else { format!("┴{}", "─".repeat(w + 2)) })
        .collect::<String>() + "┘";
    state.mp.println(format!("  {DIM}{top}{RESET}")).ok();
    for (row_i, row) in rendered.iter().enumerate() {
        let cells: String = row.iter().enumerate().map(|(c, cell)| {
            let pad = " ".repeat(col_widths[c].saturating_sub(visible_width(cell)));
            format!("{DIM}│{RESET} {cell}{pad} ")
        }).collect::<String>() + &format!("{DIM}│{RESET}");
        state.mp.println(format!("  {cells}")).ok();
        if row_i == 0 && rows.len() > 1 {
            state.mp.println(format!("  {DIM}{mid}{RESET}")).ok();
        }
    }
    state.mp.println(format!("  {DIM}{bot}{RESET}")).ok();
    state.mp.println(String::new()).ok();
}
