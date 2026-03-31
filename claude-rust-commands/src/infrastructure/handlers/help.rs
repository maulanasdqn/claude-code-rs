use crate::domain::CommandResult;

pub fn handle_help() -> CommandResult {
    let text = [
        "Available commands:",
        "  /help     - Show this help message",
        "  /clear    - Clear conversation history",
        "  /compact  - Compact conversation to save context",
        "  /model <name> - Switch to a different model",
        "  /quit     - Exit the application",
        "  /exit     - Exit the application",
    ]
    .join("\n");

    CommandResult::Output(text)
}
