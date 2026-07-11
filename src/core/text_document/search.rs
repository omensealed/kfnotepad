pub fn repeat_search_next(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
) -> SearchRepeatResult {
    repeat_search_next_with_mode(
        document,
        cursor,
        query,
        SearchMode {
            case_sensitive: true,
        },
    )
}

pub fn repeat_search_next_with_mode(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
    mode: SearchMode,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }

    let start = super::next_search_start(document, *cursor);
    if let Some(found) = document
        .buffer
        .find_next_with_mode(query, start, mode)
        .or_else(|| {
            document
                .buffer
                .find_next_with_mode(query, Cursor { row: 0, column: 0 }, mode)
        })
    {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}

pub fn repeat_search_previous(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
) -> SearchRepeatResult {
    repeat_search_previous_with_mode(
        document,
        cursor,
        query,
        SearchMode {
            case_sensitive: true,
        },
    )
}

pub fn repeat_search_previous_with_mode(
    document: &TextDocument,
    cursor: &mut Cursor,
    query: &str,
    mode: SearchMode,
) -> SearchRepeatResult {
    if query.is_empty() {
        return SearchRepeatResult::NoPreviousSearch;
    }

    if let Some(found) = document
        .buffer
        .find_previous_with_mode(query, *cursor, mode)
    {
        *cursor = found;
        SearchRepeatResult::Found {
            query: query.to_string(),
        }
    } else {
        SearchRepeatResult::NoMatch {
            query: query.to_string(),
        }
    }
}
