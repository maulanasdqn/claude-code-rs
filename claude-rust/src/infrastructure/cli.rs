use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "claude-rust",
    about = "Interactive CLI for Claude AI",
    version,
    after_help = "EXAMPLES:\n  \
        claude-rust                          Interactive TUI\n  \
        claude-rust -p 'explain this'        One-shot query\n  \
        cat file.rs | claude-rust            Pipe mode\n  \
        claude-rust --json -p 'list files'   JSON output mode"
)]
pub struct Cli {
    #[arg(short, long, help = "One-shot prompt to send")]
    pub prompt: Option<String>,

    #[arg(short, long, help = "Override model name")]
    pub model: Option<String>,

    #[arg(long, help = "Output JSON instead of plain text")]
    pub json: bool,

    #[arg(long, help = "Maximum agentic turns")]
    pub max_turns: Option<usize>,

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
