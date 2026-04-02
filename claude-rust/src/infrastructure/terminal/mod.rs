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
mod input_clipboard;
mod input_draw;
mod input_handler;
mod input_raw;
mod input_search;
mod input_select;
mod input_shortcuts;
mod input_suggest;
mod input_vim;
mod prompt_sections;
mod system_prompt;
mod tool_display;

pub use banner::{layout_width, print_banner, term_width};
pub use git::git_branch;
pub use input::{prompt_resume, read_user_input};
pub use input_border::set_current_model;
pub use input_raw::{clear_pasted_images, take_pasted_images};
pub use input_select::select_from_list;
pub use system_prompt::{build_env_info, make_system_prompt};
pub use tool_display::{summarize_tool_input, tool_display_name};
