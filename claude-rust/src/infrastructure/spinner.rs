use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::infrastructure::terminal::{CYAN, DIM, RESET, SPINNER_VERBS};

pub fn spawn_spinner(
    mode_flag: Arc<AtomicU8>,
) -> (Arc<AtomicBool>, tokio::task::JoinHandle<()>) {
    let _ = mode_flag;
    let spinning = Arc::new(AtomicBool::new(true));
    let spinning_clone = spinning.clone();
    let handle = tokio::spawn(async move {
        const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut frame = 0;
        let mut verb_idx = 0;
        let mut ticks = 0u64;
        while spinning_clone.load(Ordering::Relaxed) {
            let verb = SPINNER_VERBS[verb_idx % SPINNER_VERBS.len()];
            let dots = match (ticks / 4) % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };
            eprint!(
                "\r  {CYAN}{}{RESET} {DIM}{verb}{dots}{RESET}\x1b[K",
                FRAMES[frame % FRAMES.len()]
            );
            io::stderr().flush().ok();
            frame += 1;
            ticks += 1;
            if ticks % 25 == 0 {
                verb_idx += 1;
            }
            tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        }
        eprint!("\r\x1b[2K");
        io::stderr().flush().ok();
    });
    (spinning, handle)
}
