pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const ORANGE: &str = "\x1b[38;5;214m";


mod banner;
mod git;
mod input;
mod input_border;
mod input_draw;
mod input_handler;
mod input_raw;
mod input_vim;
mod prompt_sections;
mod system_prompt;
mod tool_display;

pub use banner::{print_banner, term_width};
pub use input::{prompt_resume, read_user_input};
pub use system_prompt::{build_env_info, make_system_prompt};
pub use tool_display::{summarize_tool_input, tool_display_name, tool_icon};
