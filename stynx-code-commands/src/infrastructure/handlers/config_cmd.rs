use crate::domain::CommandResult;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn handle_config(settings_json: &str) -> CommandResult {
    let mut output = format!("  {BOLD}Merged Settings{RESET}\n\n");
    // Pretty-print the settings
    for line in settings_json.lines() {
        output.push_str(&format!("  {DIM}{line}{RESET}\n"));
    }
    CommandResult::Output(output)
}
