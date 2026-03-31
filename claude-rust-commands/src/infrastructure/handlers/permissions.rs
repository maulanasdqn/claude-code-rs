use crate::domain::CommandResult;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

pub fn handle_permissions(allow: &[String], deny: &[String]) -> CommandResult {
    let mut output = format!("  {BOLD}Permission Rules{RESET}\n\n");

    if allow.is_empty() && deny.is_empty() {
        output.push_str(&format!("  {DIM}No permission rules configured.{RESET}\n"));
        output.push_str(&format!("  {DIM}Add rules in ~/.claude/settings.json or .claude/settings.json{RESET}\n"));
        return CommandResult::Output(output);
    }

    if !allow.is_empty() {
        output.push_str(&format!("  {GREEN}{BOLD}Allow:{RESET}\n"));
        for rule in allow {
            output.push_str(&format!("    {GREEN}✓{RESET} {DIM}{rule}{RESET}\n"));
        }
    }

    if !deny.is_empty() {
        if !allow.is_empty() {
            output.push('\n');
        }
        output.push_str(&format!("  {RED}{BOLD}Deny:{RESET}\n"));
        for rule in deny {
            output.push_str(&format!("    {RED}✗{RESET} {DIM}{rule}{RESET}\n"));
        }
    }

    CommandResult::Output(output)
}
