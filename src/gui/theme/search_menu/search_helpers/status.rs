use super::*;

pub(in crate::gui::app::state) fn search_result_status(
    result: SearchRepeatResult,
    backwards: bool,
) -> String {
    match result {
        SearchRepeatResult::NoPreviousSearch => "search query required".to_string(),
        SearchRepeatResult::Found { query } if backwards => format!("found previous: {query}"),
        SearchRepeatResult::Found { query } => format!("found next: {query}"),
        SearchRepeatResult::NoMatch { query } => format!("no match: {query}"),
    }
}

pub(in crate::gui::app::state) fn go_to_line_status(result: GoToLineResult) -> String {
    match result {
        GoToLineResult::Empty => "Line number is empty".to_string(),
        GoToLineResult::Invalid => "Line number is invalid".to_string(),
        GoToLineResult::OutOfRange { line_number } => {
            format!("Line out of range: {line_number}")
        }
        GoToLineResult::Moved { line_number } => format!("Line {line_number}"),
    }
}
