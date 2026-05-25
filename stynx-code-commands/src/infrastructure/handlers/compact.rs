use crate::domain::CommandResult;

pub fn handle_compact() -> CommandResult {
    CommandResult::Output("Compact requires engine integration".to_string())
}
