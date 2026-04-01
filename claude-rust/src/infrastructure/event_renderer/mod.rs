mod render_cost;
mod render_diff;
mod render_error;
mod render_event;
mod render_md;
pub(super) mod render_spinner;
pub(super) mod render_syntax;

pub use render_cost::{render_cost, render_exit_summary};
pub use render_error::render_error_box;
pub use render_event::render_event;

pub struct RenderState {
    pub in_text: bool,
    pub in_thinking: bool,
    pub(super) tool_json_buf: String,
    pub(super) current_tool_name: String,
    pub(super) line_buf: String,
    pub(super) in_code_block: bool,
    pub(super) code_block_lang: String,
    pub(super) thinking_start: Option<std::time::Instant>,
    pub(super) tool_anim: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub(super) turn_input: u64,
    pub(super) turn_output: u64,
    pub(super) spinner: Option<render_spinner::SpinnerState>,
    pub(super) spin_frame: usize,
    pub(super) highlighter: Option<render_syntax::Highlighter<'static>>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            in_text: false, in_thinking: false,
            tool_json_buf: String::new(), current_tool_name: String::new(),
            line_buf: String::new(), in_code_block: false, code_block_lang: String::new(),
            thinking_start: None, tool_anim: None, turn_input: 0, turn_output: 0,
            spinner: None, spin_frame: 0, highlighter: None,
        }
    }
}
