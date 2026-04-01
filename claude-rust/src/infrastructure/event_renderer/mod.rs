mod render_cost;
mod render_error;
mod render_event;
mod render_md;

pub use render_cost::render_cost;
pub use render_error::render_error_box;
pub use render_event::render_event;

pub struct RenderState {
    pub in_text: bool,
    pub in_thinking: bool,
    pub(super) tool_json_buf: String,
    pub(super) current_tool_name: String,
    pub(super) line_buf: String,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            in_text: false,
            in_thinking: false,
            tool_json_buf: String::new(),
            current_tool_name: String::new(),
            line_buf: String::new(),
        }
    }
}
