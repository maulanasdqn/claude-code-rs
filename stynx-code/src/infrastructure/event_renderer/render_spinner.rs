use std::time::Instant;

const VERBS: &[&str] = &[
    "Thinking", "Reasoning", "Pondering", "Analyzing", "Considering",
    "Processing", "Working", "Computing", "Calculating", "Reflecting",
    "Contemplating", "Deliberating", "Examining", "Investigating", "Synthesizing",
];
const SPIN: &[&str] = &["·", "✢", "✳", "✶", "✻", "✽"];

pub struct SpinnerState {
    verb: &'static str,
    start: Instant,
    pub tokens: u64,
}

impl SpinnerState {
    pub fn new() -> Self {
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0) as usize) % VERBS.len();
        Self { verb: VERBS[idx], start: Instant::now(), tokens: 0 }
    }

    pub fn render_frame(&self, frame: usize) -> String {
        let elapsed = self.start.elapsed().as_secs();
        let spin = SPIN[frame % SPIN.len()];

        let color = if elapsed >= 15 {
            "\x1b[31m"
        } else if elapsed >= 5 {
            "\x1b[33m"
        } else {
            "\x1b[2m"
        };

        let verb = self.verb;
        let elapsed_str = if elapsed >= 3 {
            format!("  \x1b[2m{elapsed}s\x1b[0m")
        } else {
            String::new()
        };
        let token_str = if elapsed >= 30 && self.tokens > 0 {
            let t = if self.tokens >= 1000 {
                format!("{:.1}K", self.tokens as f64 / 1000.0)
            } else {
                self.tokens.to_string()
            };
            format!("  \x1b[2m↓ {t}\x1b[0m")
        } else {
            String::new()
        };

        format!("  {color}{spin}  {verb}…\x1b[0m{elapsed_str}{token_str}")
    }
}
