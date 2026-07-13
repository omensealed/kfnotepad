use super::*;

#[test]
fn parses_help_flag() {
    assert_eq!(parse_args(&["--help".to_string()]), Ok(Command::Help));
}

#[test]
fn parses_managed_notes_flags() {
    assert_eq!(
        parse_args(&["--notes".to_string()]),
        Ok(Command::ListManagedNotes)
    );
    assert_eq!(
        parse_args(&["--note".to_string(), "Daily Note".to_string()]),
        Ok(Command::OpenManagedNote("Daily Note".to_string()))
    );
    assert_eq!(
        parse_args(&["--note".to_string(), "   ".to_string()]),
        Err("managed note title must not be empty".to_string())
    );
}

#[test]
fn rejects_unknown_option() {
    assert_eq!(
        parse_args(&["--bogus".to_string()]),
        Err("unknown option: --bogus".to_string())
    );
}

#[test]
fn summarizes_text_without_mutation() {
    assert_eq!(
        summarize_text("one\ntwo\n"),
        FileSummary {
            bytes: 8,
            lines: 2,
            trailing_newline: true
        }
    );
}
