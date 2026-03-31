use crate::application::expand_file_references;

#[test]
fn no_references_unchanged() {
    let input = "hello world";
    assert_eq!(expand_file_references(input), "hello world");
}

#[test]
fn email_not_expanded() {
    let input = "contact user@example.com please";
    assert_eq!(expand_file_references(input), input);
}

#[test]
fn nonexistent_file_left_as_is() {
    let input = "look at @nonexistent_file_abc123.txt";
    assert_eq!(expand_file_references(input), input);
}
