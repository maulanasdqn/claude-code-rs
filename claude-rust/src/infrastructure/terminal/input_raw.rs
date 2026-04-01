use std::sync::{Arc, atomic::AtomicU8};

use crossterm::event;

use super::input_handler::process_key;

pub(super) fn read_line_raw(
    inner_width: usize,
    mode: &Arc<AtomicU8>,
    history: &[String],
    skill_names: &[String],
) -> Option<String> {
    let mut buf = String::new();
    let mut cursor_pos: usize = 0;
    let mut hist_idx: Option<usize> = None;
    let mut saved_buf = String::new();
    let mut extra_lines: usize = 0;
    let mut vim_mode: bool = false;

    loop {
        if !event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => return None,
        };
        if let Some(result) = process_key(
            ev,
            &mut buf,
            &mut cursor_pos,
            &mut hist_idx,
            &mut saved_buf,
            history,
            skill_names,
            inner_width,
            mode,
            &mut extra_lines,
            &mut vim_mode,
        ) {
            return result;
        }
    }
}
