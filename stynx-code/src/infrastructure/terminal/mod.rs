pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";

pub mod banner;
pub mod input;
pub mod input_border;
pub mod input_select;

pub use banner::{layout_width, print_banner, term_width};
pub use input::prompt_resume;
pub use input_border::set_current_model;
pub use input_select::select_from_list;
pub use stynx_code_app::{build_env_info, git_branch, make_system_prompt};
