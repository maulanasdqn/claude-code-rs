use claude_rust_commands::expand_message_content;
use claude_rust_types::{Conversation, Message};

use crate::output::{print_prompt, render_error};
use crate::runner::QueryRunner;
use crate::setup::AppContext;

pub async fn run_repl(ctx: &AppContext, system_prompt: String) {
    let mut conversation = Conversation {
        system: Some(system_prompt),
        ..Default::default()
    };

    let runner = QueryRunner::new(ctx.engine.clone(), false);
    let stdin = std::io::stdin();

    loop {
        print_prompt();
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                render_error(&format!("stdin read: {e}"));
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed {
            "/quit" | "/exit" | "/q" => break,
            "/clear" => {
                conversation.messages.clear();
                eprintln!("  conversation cleared");
                continue;
            }
            "/help" => {
                print_repl_help();
                continue;
            }
            _ => {}
        }

        let content = expand_message_content(trimmed);
        conversation.push(Message {
            role: claude_rust_types::Role::User,
            content,
        });

        match runner.run(conversation.clone()).await {
            Ok(result) => {
                conversation = result.conversation;
            }
            Err(e) => {
                render_error(&e);
                conversation.messages.pop();
            }
        }
    }
}

fn print_repl_help() {
    eprintln!();
    eprintln!("  Commands:");
    eprintln!("    /help    Show this help");
    eprintln!("    /clear   Clear conversation history");
    eprintln!("    /quit    Exit the REPL");
    eprintln!();
}
