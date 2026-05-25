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
         {DIM}  /usage{RESET}             {DIM}Show plan usage limits{RESET}\n\
         {DIM}  /model{RESET} {DIM}<name>{RESET}      {DIM}Switch model{RESET}\n\
         {DIM}  /fast{RESET}              {DIM}Toggle fast (haiku) model{RESET}\n\
         {DIM}  /effort{RESET} {DIM}<level>{RESET}    {DIM}Set effort (low/medium/high/max){RESET}\n\
         {DIM}  /diff{RESET}              {DIM}Show git diff{RESET}\n\
         {DIM}  /status{RESET}            {DIM}Show git status{RESET}\n\
         {DIM}  /review{RESET}            {DIM}Review the current git diff{RESET}\n\
         {DIM}  /commit{RESET}            {DIM}Generate git commit message{RESET}\n\
         {DIM}  /doctor{RESET}            {DIM}Check environment health{RESET}\n\
         {DIM}  /config{RESET}            {DIM}Show merged settings{RESET}\n\
         {DIM}  /permissions{RESET}       {DIM}Show allow/deny rules{RESET}\n\
         {DIM}  /session{RESET} {DIM}[id]{RESET}      {DIM}List or load sessions{RESET}\n\
         {DIM}  /memory{RESET}            {DIM}Show CLAUDE.md / MEMORY.md{RESET}\n\
         {DIM}  /export{RESET}            {DIM}Export conversation to markdown{RESET}\n\
         {DIM}  /rewind{RESET} {DIM}[n]{RESET}        {DIM}Remove last n exchanges (default 1){RESET}\n\
         {DIM}  /plan{RESET}              {DIM}Toggle plan mode{RESET}\n\
         {DIM}  /mode{RESET}              {DIM}Cycle permission mode{RESET}\n\
         {DIM}  /think{RESET}             {DIM}Toggle extended thinking{RESET}\n\
         {DIM}  /init{RESET}              {DIM}Create/update CLAUDE.md{RESET}\n\
         {DIM}  /add{RESET} {DIM}<path>{RESET}        {DIM}Pin file to context{RESET}\n\
         {DIM}  /files{RESET}             {DIM}Show pinned files{RESET}\n\
         {DIM}  /copy{RESET}              {DIM}Copy last response to clipboard{RESET}\n\
         {DIM}  /login{RESET}             {DIM}Show authentication instructions{RESET}\n\
         {DIM}  /logout{RESET}            {DIM}Clear saved credentials{RESET}\n\
         {DIM}  /vim{RESET}               {DIM}Show vim mode status{RESET}\n\
         {DIM}  /version{RESET}           {DIM}Show stynx-code version{RESET}\n\
         {DIM}  /quit{RESET}              {DIM}Exit{RESET}\n\
         \n\
         {DIM}  !<command>{RESET}         {DIM}Run shell command directly{RESET}\n\
         \n\
         {DIM}  Tip: Use @file_path to include file contents in your message.{RESET}\n\
         \n\
         {DIM}  Tools (used by the assistant){RESET}\n\
         {DIM}  agent{RESET}              {DIM}General sub-agent (all tools){RESET}\n\
         {DIM}  explore{RESET}            {DIM}Read-only exploration sub-agent{RESET}"
    );

    CommandResult::Output(text)
}
