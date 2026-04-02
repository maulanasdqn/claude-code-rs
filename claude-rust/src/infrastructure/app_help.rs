use super::skills::Skill;
use super::terminal::{BOLD, CYAN, DIM, RESET, layout_width};

pub fn print_help(skills: &[Skill]) {
    let w = layout_width().min(72);
    println!();
    println!("  {BOLD}{CYAN}Slash Commands{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(w));

    let cmds: &[(&str, &str)] = &[
        ("/help",        "Show this help"),
        ("/clear",       "Clear conversation history"),
        ("/compact",     "Compact context to save tokens"),
        ("/cost",        "Show token usage and cost"),
        ("/usage",       "Show plan usage limits (OAuth only)"),
        ("/model [name]","Switch model  (aliases: opus/sonnet/haiku)"),
        ("/fast",        "Toggle fast mode (haiku)"),
        ("/think",       "Toggle extended thinking"),
        ("/effort",      "Set effort level (low/medium/high/max)"),
        ("/mode",        "Cycle permission mode"),
        ("/plan [task]", "Toggle plan mode, or plan a task"),
        ("/conductor <task>", "Orchestrate parallel worker agents"),
        ("/reflect",          "Toggle self-correction (default: off)"),
        ("/diff",        "Show git diff"),
        ("/status",      "Show git status"),
        ("/review",      "Review current git diff"),
        ("/commit",      "Generate a commit message"),
        ("/memory",      "Show CLAUDE.md"),
        ("/init",        "Create/update CLAUDE.md for this project"),
        ("/add <path>",  "Pin a file to every message"),
        ("/files",       "List pinned files"),
        ("/skills",      "List available skills"),
        ("/session",     "List or load previous sessions"),
        ("/rewind [n]",  "Remove last n exchanges (default 1)"),
        ("/export",      "Export conversation to markdown"),
        ("/copy",        "Copy last response to clipboard"),
        ("/undo [n]",    "Restore last n file edits"),
        ("/config",      "Show merged config"),
        ("/permissions", "Show allow/deny rules"),
        ("/version",     "Show version"),
        ("/vim",         "Vim mode status"),
        ("/login",       "Authentication instructions"),
        ("/logout",      "Clear stored credentials"),
        ("/quit",        "Exit session"),
    ];

    for (cmd, desc) in cmds {
        println!("  {CYAN}{cmd:<22}{RESET}  {DIM}{desc}{RESET}");
    }

    if !skills.is_empty() {
        println!();
        println!("  {BOLD}{CYAN}Skills{RESET}");
        println!("  {DIM}{}{RESET}", "─".repeat(w));
        for s in skills {
            if s.user_invocable {
                let hint = s.argument_hint.as_deref().map(|h| format!(" {h}")).unwrap_or_default();
                println!("  {CYAN}/{:<22}{RESET}  {DIM}{}{RESET}", format!("{}{hint}", s.name), s.description);
            }
        }
    }

    println!();
    println!("  {BOLD}{CYAN}Keyboard Shortcuts{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(w));

    let shortcuts: &[(&str, &str)] = &[
        ("Enter",        "Submit message"),
        ("Alt+Enter",    "Insert newline"),
        ("Shift+Tab",    "Cycle permission mode"),
        ("Ctrl+C / Esc", "Interrupt current operation"),
        ("Ctrl+R",       "History search"),
        ("Ctrl+G",       "Open message in $EDITOR"),
        ("Ctrl+V",       "Paste image from clipboard"),
        ("Ctrl+B",       "Send to background task"),
        ("Ctrl+S",       "Stash / restore input buffer"),
        ("Ctrl+L",       "Clear screen"),
        ("Ctrl+U",       "Clear input line"),
        ("Ctrl+W",       "Delete previous word"),
        ("Ctrl+A / Home","Move to start of line"),
        ("Ctrl+E / End", "Move to end of line"),
        ("Ctrl+O",       "Show transcript"),
        ("Ctrl+T",       "Show todos"),
        ("Alt+P",        "Model picker"),
        ("Alt+O",        "Toggle fast mode"),
        ("Alt+T",        "Toggle thinking"),
        ("Esc",          "Enter vim normal mode"),
        ("!<command>",   "Run shell command"),
    ];

    for (key, desc) in shortcuts {
        println!("  {CYAN}{key:<24}{RESET}  {DIM}{desc}{RESET}");
    }

    println!();
}
