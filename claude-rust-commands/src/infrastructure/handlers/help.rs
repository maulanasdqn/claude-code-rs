use crate::domain::CommandResult;

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";

pub fn handle_help() -> CommandResult {
    let text = format!(
        "  {BOLD}{CYAN}Commands{RESET}\n\
         \n\
         {DIM}  ┌─────────────────────────────────────────────────────┐{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/help{RESET}            Show this help message          {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/clear{RESET}           Clear conversation history      {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/compact{RESET}         Compact context to save tokens  {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/cost{RESET}            Show token usage and cost       {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/model{RESET} {DIM}<name>{RESET}    Switch model                    {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/diff{RESET}            Show git diff                   {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/status{RESET}          Show git status                 {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/doctor{RESET}          Check environment health        {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/config{RESET}          Show merged settings            {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/permissions{RESET}     Show allow/deny rules           {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/session{RESET} {DIM}[id]{RESET}   List or load sessions           {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/plan{RESET}            Toggle plan mode                {DIM}│{RESET}\n\
         {DIM}  │{RESET}  {BOLD}/quit{RESET}            Exit                            {DIM}│{RESET}\n\
         {DIM}  └─────────────────────────────────────────────────────┘{RESET}\n\
         \n\
         {DIM}  Tip: Use @file_path to include file contents in your message.{RESET}"
    );

    CommandResult::Output(text)
}
