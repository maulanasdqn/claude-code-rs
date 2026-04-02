mod render_cost;
mod render_diff;
mod render_error;
mod render_event;
mod render_md;
pub(super) mod render_syntax;

pub use render_cost::{render_cost, render_exit_summary};
pub use render_error::render_error_box;
pub use render_event::render_event;

pub struct RenderState {
    pub in_text: bool,
    pub in_thinking: bool,
    pub(super) text_started: bool,
    pub(super) line_buf: String,
    pub(super) in_code_block: bool,
    pub(super) code_block_lang: String,
    pub(super) turn_input: u64,
    pub(super) turn_output: u64,
    pub(super) highlighter: Option<render_syntax::Highlighter<'static>>,
    pub mp: indicatif::MultiProgress,
    /// Single persistent spinner for all tool activity — updated in-place, never stacks.
    pub(super) activity_pb: Option<indicatif::ProgressBar>,
    /// Tool call queue: (name, input_json). No spinner stored here.
    pub(super) active_tools: std::collections::VecDeque<(String, String)>,
    pub(super) current_json_buf: String,
    pub(super) thinking_pb: Option<indicatif::ProgressBar>,
    /// Deduplication for consecutive reads/searches to the same target.
    /// (display_label, count, lines)
    pub(super) last_read: Option<(String, usize, usize)>,
    /// Buffered table rows (non-separator `|` lines) waiting to be rendered.
    pub(super) table_rows: Vec<Vec<String>>,
}

impl RenderState {
    pub fn new(mp: indicatif::MultiProgress) -> Self {
        Self {
            in_text: false,
            in_thinking: false,
            text_started: false,
            line_buf: String::new(),
            in_code_block: false,
            code_block_lang: String::new(),
            turn_input: 0,
            turn_output: 0,
            highlighter: None,
            mp,
            activity_pb: None,
            active_tools: std::collections::VecDeque::new(),
            current_json_buf: String::new(),
            thinking_pb: None,
            last_read: None,
            table_rows: Vec::new(),
        }
    }
}
