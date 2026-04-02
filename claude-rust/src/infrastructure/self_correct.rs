/// Self-correction pass.
///
/// After every successful engine turn, an independent reflection engine
/// evaluates the assistant's response against the standards and skills
/// defined in the system prompt. If it finds a violation it returns the
/// corrected response, which replaces the original in the conversation.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use claude_rust_engine::{EngineEvent, QueryEngine};
use claude_rust_types::{ContentBlock, Conversation, Message, Role};

use super::terminal::{BOLD, CYAN, DIM, RESET};

const TICKS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// System prompt for the reflection engine.
fn reflection_system(system_prompt: &str) -> String {
    format!(
        "You are a strict quality-control evaluator for an AI coding assistant.\n\
         \n\
         Your task: decide whether the assistant's response correctly follows ALL \
         standards, rules, skills, and tone guidelines stated in the system prompt below.\n\
         \n\
         ── SYSTEM PROMPT ────────────────────────────────────────────────\n\
         {system_prompt}\n\
         ─────────────────────────────────────────────────────────────────\n\
         \n\
         Evaluation checklist:\n\
         1. Does the response obey every rule in the system prompt (no filler, no \
            summaries, no unsolicited features, correct tone, etc.)?\n\
         2. If a skill/template was invoked, was it applied correctly and completely?\n\
         3. Is every code change correct, minimal, and free from introduced bugs?\n\
         4. Is the response complete — no placeholders, no \"TODO\", no half-finished work?\n\
         \n\
         Your reply:\n\
         • If the response is fully correct → reply with ONLY the single character: ✓\n\
         • If anything needs fixing → reply with the corrected response ONLY. \
           No preamble, no explanation — just the corrected content starting immediately.\n\
         • Do NOT repeat the user's question. Do NOT narrate what you changed.\n\
         • If you output a correction, it must be complete and ready to show to the user."
    )
}

/// Build the user-facing evaluation request.
fn reflection_query(user_msg: &str, assistant_msg: &str) -> String {
    format!(
        "── USER REQUEST ─────────────────────────────────────────────────\n\
         {user_msg}\n\
         \n\
         ── ASSISTANT RESPONSE ───────────────────────────────────────────\n\
         {assistant_msg}\n\
         ─────────────────────────────────────────────────────────────────\n\
         \n\
         Does the assistant's response meet all standards? If yes: ✓\n\
         If not: provide the corrected response."
    )
}

/// Run one self-correction pass.
///
/// Returns `true` if a correction was applied (the last assistant message
/// in `conversation` is replaced with the corrected version).
pub async fn run_self_correct(
    conversation: &mut Conversation,
    reflect_engine: &Arc<QueryEngine>,
    system_prompt: &str,
) -> bool {
    // Extract the last user message and last assistant text from the conversation.
    let user_msg = last_text_of_role(conversation, Role::User);
    let assistant_msg = last_text_of_role(conversation, Role::Assistant);

    // Skip if there is nothing to evaluate.
    if user_msg.is_empty() || assistant_msg.is_empty() {
        return false;
    }

    // Skip trivial single-word / ✓-like assistant replies to avoid infinite loops.
    if assistant_msg.trim().len() < 4 {
        return false;
    }

    // Show a brief spinner during reflection.
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.dim}  {msg:.dim}")
            .unwrap()
            .tick_strings(TICKS),
    );
    pb.set_message("Self-checking…");
    pb.enable_steady_tick(Duration::from_millis(150));

    let mut reflect_conv = Conversation {
        system: Some(reflection_system(system_prompt)),
        ..Default::default()
    };
    reflect_conv.push(Message {
        role: Role::User,
        content: vec![ContentBlock::Text {
            text: reflection_query(&user_msg, &assistant_msg),
        }],
    });

    let output = Arc::new(Mutex::new(String::new()));
    let out_ref = output.clone();

    let result = reflect_engine
        .run(reflect_conv, move |ev| {
            if let EngineEvent::TextDelta(t) = ev {
                out_ref.lock().unwrap().push_str(&t);
            }
        })
        .await;

    pb.finish_and_clear();

    if result.is_err() {
        return false;
    }

    let correction = output.lock().unwrap().trim().to_string();

    // "✓" (or starts with ✓) means the response was correct.
    if correction.starts_with('✓') || correction.is_empty() {
        return false;
    }

    // Replace the last assistant message with the corrected content.
    if let Some(last) = conversation.messages.iter_mut().rev()
        .find(|m| m.role == Role::Assistant)
    {
        last.content = vec![ContentBlock::Text { text: correction.clone() }];
    }

    // Render the corrected response.
    println!("\n  {CYAN}{BOLD}◆  Self-corrected{RESET}  {DIM}────────────────────────────────{RESET}\n");
    for line in correction.lines() {
        println!("  {line}");
    }
    println!();

    true
}

/// Extract all text blocks from the last message with the given role.
fn last_text_of_role(conv: &Conversation, role: Role) -> String {
    conv.messages.iter().rev()
        .find(|m| m.role == role)
        .map(|m| {
            m.content.iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
