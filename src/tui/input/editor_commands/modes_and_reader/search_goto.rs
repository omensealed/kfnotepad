//! Search repetition, go-to-line status, and mode activation.

use super::*;

pub(crate) fn start_search(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.goto_line_active = false;
    runtime.goto_line_query.clear();
    runtime.search_active = true;
    runtime.search_query.clear();
    runtime.search_history_index = None;
    runtime.status = String::from("Search: ");
}

pub(crate) fn repeat_search(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    let query = runtime.last_search_query.clone();
    runtime.status = search_repeat_status(repeat_search_next_with_mode(
        document,
        cursor,
        &query,
        current_search_mode(runtime),
    ));
}

pub(crate) fn repeat_search_previous(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
) {
    runtime.quit_confirmation_pending = false;
    let query = runtime.last_search_query.clone();
    runtime.status = search_repeat_status(repeat_search_previous_with_mode(
        document,
        cursor,
        &query,
        current_search_mode(runtime),
    ));
}

pub(crate) fn search_repeat_status(result: SearchRepeatResult) -> String {
    match result {
        SearchRepeatResult::NoPreviousSearch => String::from("No previous search"),
        SearchRepeatResult::Found { query } => format!("Found: {query}"),
        SearchRepeatResult::NoMatch { query } => format!("No match: {query}"),
    }
}

pub(crate) fn go_to_line_status(result: GoToLineResult) -> String {
    match result {
        GoToLineResult::Empty => String::from("Line number is empty"),
        GoToLineResult::Invalid => String::from("Line number is invalid"),
        GoToLineResult::OutOfRange { line_number } => {
            format!("Line out of range: {line_number}")
        }
        GoToLineResult::Moved { line_number } => format!("Line {line_number}"),
    }
}

pub(crate) fn start_goto_line(runtime: &mut EditorRuntime) {
    runtime.quit_confirmation_pending = false;
    runtime.search_active = false;
    runtime.search_query.clear();
    runtime.goto_line_active = true;
    runtime.goto_line_query.clear();
    runtime.status = String::from("Go to line: ");
}

pub(crate) fn current_search_mode(runtime: &EditorRuntime) -> SearchMode {
    SearchMode {
        case_sensitive: runtime.settings.search_case_sensitive,
    }
}
