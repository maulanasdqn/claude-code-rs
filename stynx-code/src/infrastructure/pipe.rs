use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use stynx_code_commands::expand_message_content;
use stynx_code_engine::QueryEngine;
use stynx_code_types::{ContentBlock, Conversation, EngineEvent, Message, Role};

use super::event_renderer::render_stream_event;

pub async fn run_pipe(
    engine: &Arc<QueryEngine>,
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
        messages: vec![Message { role: Role::User, content }],
    };

    let total_input = Arc::new(AtomicU64::new(0));
    let total_output = Arc::new(AtomicU64::new(0));

    let ti = total_input.clone();
    let to = total_output.clone();

    let result = engine
        .run(conversation, move |event| {
            if let EngineEvent::Usage { input_tokens, output_tokens } = &event {
                if *input_tokens > 0 { ti.fetch_add(*input_tokens, Ordering::Relaxed); }
                if *output_tokens > 0 { to.fetch_add(*output_tokens, Ordering::Relaxed); }
            }
            render_stream_event(&event, json_mode);
        })
        .await
        .map_err(|e| e.to_string())?;

    if json_mode {
        let final_text = result.messages.last()
            .map(|m| extract_text(&m.content))
            .unwrap_or_default();
        let output = serde_json::json!({
            "response": final_text,
            "input_tokens": total_input.load(Ordering::Relaxed),
            "output_tokens": total_output.load(Ordering::Relaxed),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    }

    Ok(())
}

fn extract_text(blocks: &[ContentBlock]) -> String {
    blocks.iter().filter_map(|b| {
        if let ContentBlock::Text { text } = b { Some(text.as_str()) } else { None }
    }).collect::<Vec<_>>().join("")
}
