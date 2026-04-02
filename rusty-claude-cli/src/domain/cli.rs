use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "rusty-claude",
    about = "Lightweight CLI for Claude AI",
    version,
    after_help = "EXAMPLES:\n  \
        rusty-claude                     Interactive REPL\n  \
        rusty-claude -p 'explain this'   One-shot query\n  \
        cat file.rs | rusty-claude       Pipe mode\n  \
        rusty-claude --json              JSON output mode"
)]
pub struct Cli {
    #[arg(short, long, help = "One-shot prompt to send")]
    pub prompt: Option<String>,

    #[arg(short, long, help = "Override model name")]
    pub model: Option<String>,

    #[arg(long, help = "Output JSON instead of plain text")]
    pub json: bool,

    #[arg(long, help = "Maximum agentic turns", default_value = "20")]
    pub max_turns: usize,

    #[arg(long, help = "System prompt override")]
    pub system: Option<String>,

    #[arg(long, help = "Enable verbose logging")]
    pub verbose: bool,
}

impl Cli {
    pub fn is_piped() -> bool {
        !std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}
