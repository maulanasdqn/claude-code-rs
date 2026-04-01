use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering}};
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_errors::AppError;
use claude_rust_types::Conversation;

use super::event_renderer::{RenderState, render_event};
use super::terminal::{DIM, RESET};

const TICKS: &[&str] = &["·", "✢", "✳", "✶", "✻", "✽"];

pub(super) async fn run_engine(
    engine: &Arc<QueryEngine>,
    conversation: Conversation,
    total_input: &Arc<AtomicU64>,
    total_output: &Arc<AtomicU64>,
    mode_flag: &Arc<AtomicU8>,
    model_id: &str,
    cwd: &str,
    perm_paused: &Arc<AtomicBool>,
) -> Result<Conversation, AppError> {
    let _ = (mode_flag, model_id, cwd);

    let mp = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
    let init_pb = mp.add(ProgressBar::new_spinner());
    init_pb.set_style(
        ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}  {elapsed:.dim}")
            .unwrap()
            .tick_strings(TICKS),
    );
    init_pb.set_message("Thinking…");
    init_pb.enable_steady_tick(Duration::from_millis(150));

    let mut render_state = RenderState::new(mp);

    let in_counter = total_input.clone();
    let out_counter = total_output.clone();
    let cleared = Arc::new(AtomicBool::new(false));
    let cleared2 = cleared.clone();

    let engine_fut = engine.run(conversation, move |event| {
        if !cleared2.swap(true, Ordering::Relaxed) {
            init_pb.finish_and_clear();
        }
        if let EngineEvent::Usage { input_tokens, output_tokens } = &event {
            if *input_tokens > 0 { in_counter.fetch_add(*input_tokens, Ordering::Relaxed); }
            if *output_tokens > 0 { out_counter.fetch_add(*output_tokens, Ordering::Relaxed); }
        }
        render_event(event, &mut render_state);
    });

    let (esc_tx, esc_rx) = tokio::sync::oneshot::channel::<()>();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let paused2 = perm_paused.clone();

    let esc_handle = tokio::task::spawn_blocking(move || {
        crossterm::terminal::enable_raw_mode().ok();
        let mut esc_tx = Some(esc_tx);
        loop {
            if stop2.load(Ordering::Relaxed) { break; }
            if paused2.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            if crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
                    if k.code == crossterm::event::KeyCode::Esc {
                        if let Some(tx) = esc_tx.take() { let _ = tx.send(()); }
                        break;
                    }
                }
            }
        }
        crossterm::terminal::disable_raw_mode().ok();
    });

    enum Outcome { Engine(Result<Conversation, AppError>), Interrupted }

    let start = std::time::Instant::now();
    let outcome = tokio::select! {
        r = engine_fut => Outcome::Engine(r),
        _ = tokio::signal::ctrl_c() => Outcome::Interrupted,
        _ = esc_rx => Outcome::Interrupted,
    };

    stop.store(true, Ordering::Relaxed);
    esc_handle.await.ok();
    crossterm::terminal::disable_raw_mode().ok();

    match outcome {
        Outcome::Engine(r) => {
            if r.is_ok() {
                let elapsed = start.elapsed().as_secs();
                if elapsed >= 10 { notify_done(elapsed); }
            }
            r
        }
        Outcome::Interrupted => show_interrupt(&cleared),
    }
}

fn notify_done(secs: u64) {
    let msg = format!("Task completed in {secs}s");
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("osascript")
        .args(["-e", &format!("display notification \"{msg}\" with title \"Claude Code\"")])
        .output();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("notify-send")
        .args(["Claude Code", &msg, "-t", "3000"])
        .output();
}

fn show_interrupt(cleared: &AtomicBool) -> Result<Conversation, AppError> {
    if cleared.load(Ordering::Relaxed) {
        println!("\n  {DIM}^C{RESET}\n");
    } else {
        println!("  {DIM}^C{RESET}\n");
    }
    Err(AppError::Interrupted)
}
