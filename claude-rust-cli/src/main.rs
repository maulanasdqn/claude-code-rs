mod application;
mod domain;
mod infrastructure;

use clap::Parser;

use application::{build_context, run_loop, run_oneshot, run_pipe};
use domain::Cli;
use infrastructure::event_renderer::render_error_box;
use infrastructure::terminal::{build_system_prompt, print_banner};

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
            render_error_box(&e);
            std::process::exit(1);
        }
    };

    let system_prompt = cli
        .system
        .clone()
        .unwrap_or_else(|| build_system_prompt(&ctx.cwd, &ctx.provider.model_name()));

    if Cli::is_piped() {
        if let Err(e) = run_pipe(&ctx, system_prompt, cli.prompt.as_deref(), cli.json).await {
            render_error_box(&e);
            std::process::exit(1);
        }
        return;
    }

    if let Some(ref prompt) = cli.prompt {
        if let Err(e) = run_oneshot(&ctx, system_prompt, prompt, cli.json).await {
            render_error_box(&e);
            std::process::exit(1);
        }
        return;
    }

    let model_id = ctx.provider.model_name();
    print_banner(&ctx.cwd, &model_id);

    let conversation = claude_rust_types::Conversation {
        system: Some(system_prompt.clone()),
        ..Default::default()
    };

    run_loop(&ctx, system_prompt, conversation).await;
}
