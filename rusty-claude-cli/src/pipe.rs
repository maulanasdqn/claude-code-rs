use std::io::Read;

use claude_rust_commands::expand_message_content;
use claude_rust_types::{Conversation, Message, Role};

use crate::runner::QueryRunner;
use crate::setup::AppContext;

pub async fn run_pipe(
    ctx: &AppContext,
    system_prompt: String,
    extra_prompt: Option<&str>,
    json_mode: bool,
) -> Result<(), String> {
    let mut stdin_buf = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin_buf)
        .map_err(|e| format!("failed to read stdin: {e}"))?;

    let prompt = match extra_prompt {
        Some(p) => format!("{p}\n\n```\n{stdin_buf}\n```"),
        None => stdin_buf,
    };

    let content = expand_message_content(&prompt);
    let conversation = Conversation {
        system: Some(system_prompt),
        messages: vec![Message {
            role: Role::User,
            content,
        }],
    };

    let runner = QueryRunner::new(ctx.engine.clone(), json_mode);
    let result = runner.run(conversation).await?;

    if json_mode {
        let output = serde_json::json!({
            "response": result.final_text,
            "input_tokens": result.input_tokens,
            "output_tokens": result.output_tokens,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    }

    Ok(())
}
