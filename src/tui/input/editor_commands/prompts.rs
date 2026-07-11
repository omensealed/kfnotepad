pub(crate) fn handle_search_key_event(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match (event.modifiers, event.code) {
        (modifiers, KeyCode::Char('f') | KeyCode::Char('F'))
            if modifiers.contains(KeyModifiers::CONTROL)
                && modifiers.contains(KeyModifiers::SHIFT) =>
        {
            toggle_search_case(runtime);
            runtime.search_active = true;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        (_, KeyCode::Esc) => {
            runtime.search_active = false;
            runtime.search_history_index = None;
            runtime.status = String::from("Search canceled");
        }
        (_, KeyCode::Enter) => {
            if runtime.search_query.is_empty() {
                runtime.status = String::from("Search query is empty");
            } else {
                runtime.last_search_query = runtime.search_query.clone();
                remember_search_query(runtime);
                if let Some(found) = document.buffer.find_next_with_mode(
                    &runtime.search_query,
                    *cursor,
                    current_search_mode(runtime),
                ) {
                    *cursor = found;
                    runtime.status = format!("Found: {}", runtime.search_query);
                } else {
                    runtime.status = format!("No match: {}", runtime.search_query);
                }
            }
            runtime.search_active = false;
            runtime.search_history_index = None;
        }
        (_, KeyCode::Up) => recall_previous_search_history(runtime),
        (_, KeyCode::Down) => recall_next_search_history(runtime),
        (_, KeyCode::Backspace) => {
            runtime.search_query.pop();
            runtime.search_history_index = None;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value)) => {
            runtime.search_query.push(value);
            runtime.search_history_index = None;
            runtime.status = format!("Search: {}", runtime.search_query);
        }
        _ => {}
    }
}

pub(crate) fn remember_search_query(runtime: &mut EditorRuntime) {
    let query = runtime.search_query.trim();
    if query.is_empty() {
        return;
    }
    if let Some(existing) = runtime.search_history.iter().position(|item| item == query) {
        runtime.search_history.remove(existing);
    }
    runtime.search_history.insert(0, query.to_string());
    runtime.search_history.truncate(10);
}

pub(crate) fn recall_previous_search_history(runtime: &mut EditorRuntime) {
    if runtime.search_history.is_empty() {
        runtime.status = String::from("Search history empty");
        return;
    }
    let next_index = runtime
        .search_history_index
        .map_or(0, |index| (index + 1).min(runtime.search_history.len() - 1));
    runtime.search_history_index = Some(next_index);
    runtime.search_query = runtime.search_history[next_index].clone();
    runtime.status = format!("Search: {}", runtime.search_query);
}

pub(crate) fn recall_next_search_history(runtime: &mut EditorRuntime) {
    let Some(index) = runtime.search_history_index else {
        runtime.status = format!("Search: {}", runtime.search_query);
        return;
    };
    if index == 0 {
        runtime.search_history_index = None;
        runtime.search_query.clear();
    } else {
        let next_index = index - 1;
        runtime.search_history_index = Some(next_index);
        runtime.search_query = runtime.search_history[next_index].clone();
    }
    runtime.status = format!("Search: {}", runtime.search_query);
}

pub(crate) fn handle_goto_line_key_event(
    document: &TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) {
    match (event.modifiers, event.code) {
        (_, KeyCode::Esc) => {
            runtime.goto_line_active = false;
            runtime.status = String::from("Go to line canceled");
        }
        (_, KeyCode::Enter) => {
            runtime.status = go_to_line_status(shared_go_to_line(
                document,
                cursor,
                &runtime.goto_line_query,
            ));
            runtime.goto_line_active = false;
        }
        (_, KeyCode::Backspace) => {
            runtime.goto_line_query.pop();
            runtime.status = format!("Go to line: {}", runtime.goto_line_query);
        }
        (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(value))
            if value.is_ascii_digit() =>
        {
            runtime.goto_line_query.push(value);
            runtime.status = format!("Go to line: {}", runtime.goto_line_query);
        }
        _ => {}
    }
}

pub(crate) fn move_cursor(document: &TextDocument, cursor: &mut Cursor, direction: CursorMove) {
    move_document_cursor(document, cursor, direction);
}
