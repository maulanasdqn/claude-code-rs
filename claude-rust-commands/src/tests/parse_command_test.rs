use crate::domain::SlashCommand;
use crate::application::parse_command;

#[test]
fn parse_help() {
    assert!(matches!(parse_command("/help"), Some(SlashCommand::Help)));
}

#[test]
fn parse_clear() {
    assert!(matches!(parse_command("/clear"), Some(SlashCommand::Clear)));
}

#[test]
fn parse_compact() {
    assert!(matches!(
        parse_command("/compact"),
        Some(SlashCommand::Compact)
    ));
}

#[test]
fn parse_quit() {
    assert!(matches!(parse_command("/quit"), Some(SlashCommand::Quit)));
    assert!(matches!(parse_command("/exit"), Some(SlashCommand::Quit)));
}

#[test]
fn parse_model() {
    match parse_command("/model claude-3-opus") {
        Some(SlashCommand::Model(name)) => assert_eq!(name, "claude-3-opus"),
        _ => panic!("expected Model variant"),
    }
}

#[test]
fn parse_unknown() {
    assert!(parse_command("/unknown").is_none());
    assert!(parse_command("hello").is_none());
}
