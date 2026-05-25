use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use stynx_code_errors::{AppError, AppResult};
use stynx_code_types::{PermissionLevel, Tool};
use serde_json::{Value, json};

use super::question_bridge::{OptionalQuestionBridge, QuestionBridge, SharedQuestionBridge};

pub struct AskUserTool {
    paused: Arc<AtomicBool>,
    bridge: SharedQuestionBridge,
}

impl AskUserTool {
    pub fn new(paused: Arc<AtomicBool>) -> Self {
        Self { paused, bridge: Arc::new(OptionalQuestionBridge::new()) }
    }

    pub fn bridge_handle(&self) -> SharedQuestionBridge {
        self.bridge.clone()
    }

    pub fn install_bridge(&self, b: QuestionBridge) {
        self.bridge.set(b);
    }
}

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

    fn is_read_only(&self, _input: &Value) -> bool { true }
    fn requires_user_interaction(&self) -> bool { true }

    async fn execute(&self, input: Value) -> AppResult<String> {
        let question = input
            .get("question")
            .and_then(|q| q.as_str())
            .ok_or_else(|| AppError::Tool("missing 'question' field".into()))?
            .to_string();

        if let Some(bridge) = self.bridge.get() {
            return match bridge.ask(question).await {
                Some(answer) => Ok(answer),
                None => Err(AppError::Interrupted),
            };
        }

        let paused = self.paused.clone();

        let answer = tokio::task::spawn_blocking(move || {
            paused.store(true, Ordering::Relaxed);
            let result = prompt_interactive(&question);
            paused.store(false, Ordering::Relaxed);
            result
        })
        .await
        .map_err(|e| AppError::Tool(format!("ask user task failed: {e}")))??;

        Ok(answer)
    }
}

fn prompt_interactive(question: &str) -> AppResult<String> {
    use std::io::Write;
    use crossterm::terminal;

    let w = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let outer = w.saturating_sub(4).max(20);
    let inner = outer.saturating_sub(2);
    let avail = inner.saturating_sub(4);

    let top_label = "─ Question ";
    let top_fill = inner.saturating_sub(top_label.len());

    let q_lines = wrap_text(question, avail);

    let mut out = std::io::stdout();
    let _ = writeln!(out);
    let _ = writeln!(out, "  \x1b[36m\x1b[1m╭{top_label}{}\x1b[0m", "─".repeat(top_fill));
    for line in &q_lines {
        let pad = avail.saturating_sub(line.len());
        let _ = writeln!(out, "  \x1b[36m│\x1b[0m  {line}{}  \x1b[36m│\x1b[0m", " ".repeat(pad));
    }
    let sep_fill = "─".repeat(inner);
    let _ = writeln!(out, "  \x1b[36m\x1b[2m├{sep_fill}┤\x1b[0m");
    let input_pad = " ".repeat(avail);
    let _ = writeln!(out, "  \x1b[36m│\x1b[0m  \x1b[1m\x1b[36m❯\x1b[0m {input_pad}  \x1b[36m│\x1b[0m");
    let _ = writeln!(out, "  \x1b[36m\x1b[1m╰{}\x1b[0m", "─".repeat(inner));
    let input_row = q_lines.len() + 3;
    let _ = write!(out, "\x1b[{input_row}A\r\x1b[6C");
    let _ = out.flush();

    terminal::enable_raw_mode().map_err(|e| AppError::Tool(e.to_string()))?;
    let result = read_answer(avail);
    terminal::disable_raw_mode().ok();

    let lines_to_clear = q_lines.len() + 4;
    let _ = write!(out, "\x1b[{}B\r\x1b[J", lines_to_clear.saturating_sub(1));
    let _ = out.flush();

    result
}

fn read_answer(max_len: usize) -> AppResult<String> {
    use std::io::Write;
    use crossterm::event::{self, KeyCode, KeyModifiers};

    let mut buf = String::new();
    let mut out = std::io::stdout();

    loop {
        if !event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            continue;
        }
        if let event::Event::Key(k) = event::read().map_err(|e| AppError::Tool(e.to_string()))? { match (k.code, k.modifiers) {
            (KeyCode::Enter, _) => break,
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Err(AppError::Interrupted);
            }
            (KeyCode::Backspace, _) => {
                if buf.pop().is_some() {
                    let visible = buf.chars().take(max_len).collect::<String>();
                    let pad = max_len.saturating_sub(visible.len());
                    let _ = write!(out, "\r\x1b[6C{visible}{} \r\x1b[{}C",
                        " ".repeat(pad), 6 + visible.len());
                    let _ = out.flush();
                }
            }
            (KeyCode::Char(c), _) if buf.len() < max_len => {
                buf.push(c);
                let _ = write!(out, "{c}");
                let _ = out.flush();
            }
            _ => {}
        } }
    }
    Ok(buf)
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 { return vec![text.to_string()]; }
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() { lines.push(String::new()); continue; }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() { lines.push(current); }
    }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}
