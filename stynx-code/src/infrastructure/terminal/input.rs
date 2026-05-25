use std::io::{self, BufRead, Write};

use super::{BOLD, DIM, RESET};

pub fn prompt_resume() -> bool {
    eprint!("  {DIM}Previous session found. Resume? {RESET}{BOLD}[y/n]{RESET} ");
    io::stderr().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        return line.trim().starts_with('y');
    }
    false
}
