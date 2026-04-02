use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "claude-rust-cli",
    about = "Lightweight CLI for Claude AI",
    version,
    after_help = "EXAMPLES:\n  \
        claude-rust-cli                     Interactive REPL\n  \
        claude-rust-cli -p 'explain this'   One-shot query\n  \
        cat file.rs | claude-rust-cli       Pipe mode\n  \
        claude-rust-cli --json              JSON output mode"
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
