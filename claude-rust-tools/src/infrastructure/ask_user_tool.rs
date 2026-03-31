use claude_rust_errors::{AppError, AppResult};
use claude_rust_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

pub struct AskUserTool;

#[async_trait::async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &str {
        "ask_user_question"
    }

    fn description(&self) -> &str {
        "Ask the user a question and wait for their response. Use this when you need clarification or input from the user."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "The question to ask the user"
                }
            },
            "required": ["question"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let question = input
            .get("question")
            .and_then(|q| q.as_str())
            .ok_or_else(|| AppError::Tool("missing 'question' field".into()))?;

        let question = question.to_string();

        let answer = tokio::task::spawn_blocking(move || -> AppResult<String> {
            use std::io::{self, Write};

            let mut stderr = io::stderr().lock();
            write!(stderr, "\n  \x1b[1m\x1b[36m❓ {question}\x1b[0m\n  \x1b[2m➤\x1b[0m ")
                .map_err(|e| AppError::Tool(format!("failed to write question: {e}")))?;
            stderr
                .flush()
                .map_err(|e| AppError::Tool(format!("failed to flush: {e}")))?;

            let mut line = String::new();
            io::stdin()
                .read_line(&mut line)
                .map_err(|e| AppError::Tool(format!("failed to read answer: {e}")))?;

            Ok(line.trim().to_string())
        })
        .await
        .map_err(|e| AppError::Tool(format!("ask user task failed: {e}")))??;

        Ok(answer)
    }
}
