use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use claude_rust_engine::QueryEngine;
use claude_rust_types::{Conversation, ContentBlock, EngineEvent};

use crate::output::render_event;

pub struct QueryRunner {
    pub engine: Arc<QueryEngine>,
    pub json_mode: bool,
    total_input: AtomicU64,
    total_output: AtomicU64,
}

pub struct RunResult {
    pub conversation: Conversation,
    pub final_text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl QueryRunner {
    pub fn new(engine: Arc<QueryEngine>, json_mode: bool) -> Self {
        Self {
            engine,
            json_mode,
            total_input: AtomicU64::new(0),
            total_output: AtomicU64::new(0),
        }
    }

    pub async fn run(&self, mut conversation: Conversation) -> Result<RunResult, String> {
        let json_mode = self.json_mode;
        let mut collected_text = String::new();
        let text_ref = &mut collected_text;

        let result = self
            .engine
            .run(conversation.clone(), move |event| {
                if let EngineEvent::TextDelta(ref t) = event {
                    text_ref.push_str(t);
                }
                if let EngineEvent::Usage {
                    input_tokens,
                    output_tokens,
                } = &event
                {
                    let _ = input_tokens;
                    let _ = output_tokens;
                }
                render_event(&event, json_mode);
            })
            .await
            .map_err(|e| e.to_string())?;

        conversation = result;

        let final_text = conversation
            .messages
            .last()
            .map(|m| extract_text(&m.content))
            .unwrap_or_default();

        Ok(RunResult {
            conversation,
            final_text,
            input_tokens: self.total_input.load(Ordering::Relaxed),
            output_tokens: self.total_output.load(Ordering::Relaxed),
        })
    }
}

fn extract_text(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|b| {
            if let ContentBlock::Text { text } = b {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

