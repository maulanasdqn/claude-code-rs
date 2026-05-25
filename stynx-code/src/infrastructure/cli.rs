use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "stynx-code",
    about = "stynx-code — interactive AI coding assistant",
    version,
    after_help = "EXAMPLES:\n  \
        stynx-code                          Interactive TUI\n  \
        stynx-code -p 'explain this'        One-shot query\n  \
        cat file.rs | stynx-code            Pipe mode\n  \
        stynx-code --json -p 'list files'   JSON output mode"
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

    #[arg(long, help = "Thinking budget tokens (also sets max_tokens, thinking uses max_tokens - 16384)")]
    pub thinking_budget: Option<u32>,

    #[arg(long, help = "Effort level: max, high, medium, low (enables thinking + effort)")]
    pub effort: Option<String>,
}

impl Cli {
    pub fn is_piped() -> bool {
        !std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}
