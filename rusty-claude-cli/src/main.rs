mod cli;
mod oneshot;
mod output;
mod pipe;
mod repl;
mod runner;
mod setup;

use clap::Parser;

use cli::Cli;
use output::{print_banner, render_error};
use setup::build_context;

fn build_system_prompt(ctx: &setup::AppContext, custom: Option<&str>) -> String {
    match custom {
        Some(s) => s.to_string(),
        None => {
            let model = ctx.provider.model_name();
            format!(
                "You are Claude, an AI assistant by Anthropic. \
                 You help with software engineering tasks. \
                 Be concise and direct.\n\n\
                 Working directory: {}\n\
                 Model: {model}",
                ctx.cwd
            )
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let ctx = match build_context(&cli).await {
        Ok(c) => c,
        Err(e) => {
            render_error(&e);
            std::process::exit(1);
        }
    };

    let system_prompt = build_system_prompt(&ctx, cli.system.as_deref());

    if Cli::is_piped() {
        if let Err(e) =
            pipe::run_pipe(&ctx, system_prompt, cli.prompt.as_deref(), cli.json).await
        {
            render_error(&e);
            std::process::exit(1);
        }
        return;
    }

    if let Some(ref prompt) = cli.prompt {
        if let Err(e) =
            oneshot::run_oneshot(&ctx, system_prompt, prompt, cli.json).await
        {
            render_error(&e);
            std::process::exit(1);
        }
        return;
    }

    print_banner(&ctx.cwd, &ctx.provider.model_name());
    repl::run_repl(&ctx, system_prompt).await;
}
