use crate::domain::CommandResult;

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";

pub fn handle_help() -> CommandResult {
    let text = format!(
        "  {BOLD}{CYAN}Commands{RESET}\n\
         \n\
         {DIM}  /help{RESET}              {DIM}Show this help message{RESET}\n\
         {DIM}  /clear{RESET}             {DIM}Clear conversation history{RESET}\n\
         {DIM}  /compact{RESET}           {DIM}Compact context to save tokens{RESET}\n\
         {DIM}  /cost{RESET}              {DIM}Show token usage and cost{RESET}\n\
         {DIM}  /model{RESET} {DIM}<name>{RESET}      {DIM}Switch model{RESET}\n\
         {DIM}  /diff{RESET}              {DIM}Show git diff{RESET}\n\
         {DIM}  /status{RESET}            {DIM}Show git status{RESET}\n\
         {DIM}  /doctor{RESET}            {DIM}Check environment health{RESET}\n\
         {DIM}  /config{RESET}            {DIM}Show merged settings{RESET}\n\
         {DIM}  /permissions{RESET}       {DIM}Show allow/deny rules{RESET}\n\
         {DIM}  /session{RESET} {DIM}[id]{RESET}      {DIM}List or load sessions{RESET}\n\
         {DIM}  /plan{RESET}              {DIM}Toggle plan mode{RESET}\n\
         {DIM}  /mode{RESET}              {DIM}Cycle permission mode{RESET}\n\
         {DIM}  /quit{RESET}              {DIM}Exit{RESET}\n\
         \n\
         {DIM}  Tip: Use @file_path to include file contents in your message.{RESET}"
    );

    CommandResult::Output(text)
}
