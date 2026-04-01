use std::sync::{Arc, atomic::AtomicU8};

use super::input_draw::redraw_input_line;

pub(super) fn process_vim_char(
    c: char,
    buf: &mut String,
    cursor_pos: &mut usize,
    vim_mode: &mut bool,
    inner_width: usize,
    mode: &Arc<AtomicU8>,
) {
    match c {
        'h' => {
            if *cursor_pos > 0 {
                *cursor_pos -= 1;
                redraw_input_line(buf, *cursor_pos, inner_width, mode);
            }
        }
        'l' => {
            if *cursor_pos < buf.len() {
                *cursor_pos += 1;
                redraw_input_line(buf, *cursor_pos, inner_width, mode);
            }
        }
        'b' => {
            *cursor_pos = word_start(buf, *cursor_pos);
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        'w' => {
            *cursor_pos = word_end(buf, *cursor_pos);
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        'x' => {
            if *cursor_pos < buf.len() {
                buf.remove(*cursor_pos);
                redraw_input_line(buf, *cursor_pos, inner_width, mode);
            }
        }
        '0' => {
            *cursor_pos = 0;
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        '$' => {
            *cursor_pos = buf.len();
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        'i' => { *vim_mode = false; }
        'a' => {
            if *cursor_pos < buf.len() { *cursor_pos += 1; }
            *vim_mode = false;
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        'A' => {
            *cursor_pos = buf.len();
            *vim_mode = false;
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        'I' => {
            *cursor_pos = 0;
            *vim_mode = false;
            redraw_input_line(buf, *cursor_pos, inner_width, mode);
        }
        _ => {}
    }
}

fn word_start(buf: &str, pos: usize) -> usize {
    if pos == 0 { return 0; }
    let chars: Vec<char> = buf.chars().collect();
    let mut i = pos.min(chars.len()).saturating_sub(1);
    while i > 0 && chars[i].is_whitespace() { i -= 1; }
    while i > 0 && !chars[i - 1].is_whitespace() { i -= 1; }
    i
}

fn word_end(buf: &str, pos: usize) -> usize {
    let chars: Vec<char> = buf.chars().collect();
    let len = chars.len();
    if pos >= len { return len; }
    let mut i = pos;
    while i < len && chars[i].is_whitespace() { i += 1; }
    while i < len && !chars[i].is_whitespace() { i += 1; }
    i
}
