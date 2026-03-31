use crate::domain::CommandResult;

pub fn handle_model(name: &str) -> CommandResult {
    CommandResult::Output(format!("Model set to: {name}"))
}
