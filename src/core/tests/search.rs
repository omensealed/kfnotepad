use super::*;

#[test]
fn find_next_finds_query_from_cursor() {
    let buffer = TextBuffer::from_text("alpha\nbeta alphabet\n");

    assert_eq!(
        buffer.find_next("alpha", Cursor { row: 0, column: 1 }),
        Some(Cursor { row: 1, column: 5 })
    );
}

#[test]
fn find_next_wraps_to_top() {
    let buffer = TextBuffer::from_text("first match\nsecond\n");

    assert_eq!(
        buffer.find_next("first", Cursor { row: 1, column: 0 }),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_next_handles_unicode_columns() {
    let buffer = TextBuffer::from_text("aé match\n");

    assert_eq!(
        buffer.find_next("match", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_next_finds_tabs_and_emoji() {
    let buffer = TextBuffer::from_text("alpha\t🏳️‍🌈\tbeta\n");

    assert_eq!(
        buffer.find_next("🏳️‍🌈", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 6 })
    );
    assert_eq!(
        buffer.find_next("beta", Cursor { row: 0, column: 7 }),
        Some(Cursor { row: 0, column: 11 })
    );
}

#[test]
fn find_next_expands_partial_grapheme_matches_to_cluster_start() {
    let buffer = TextBuffer::from_text("🇺🇸 e\u{301}x\n");

    assert_eq!(
        buffer.find_next("🇸", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.find_next("\u{301}", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_previous_expands_partial_grapheme_matches_to_cluster_start() {
    let buffer = TextBuffer::from_text("🇺🇸 e\u{301}x\n");

    assert_eq!(
        buffer.find_previous("🇸", Cursor { row: 0, column: 2 }),
        Some(Cursor { row: 0, column: 0 })
    );
    assert_eq!(
        buffer.find_previous("\u{301}", Cursor { row: 0, column: 5 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn find_previous_handles_tabs_and_emoji() {
    let buffer = TextBuffer::from_text("alpha\t🏳️‍🌈\tbeta\n");

    assert_eq!(
        buffer.find_previous("🏳️‍🌈", Cursor { row: 0, column: 14 }),
        Some(Cursor { row: 0, column: 6 })
    );
    assert_eq!(
        buffer.find_previous("alpha", Cursor { row: 0, column: 14 }),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_next_can_ignore_case_without_changing_default_search() {
    let buffer = TextBuffer::from_text("Alpha\nbeta alpha\n");

    assert_eq!(
        buffer.find_next("alpha", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 1, column: 5 })
    );
    assert_eq!(
        buffer.find_next_with_mode(
            "alpha",
            Cursor { row: 0, column: 0 },
            SearchMode {
                case_sensitive: false,
            },
        ),
        Some(Cursor { row: 0, column: 0 })
    );
}

#[test]
fn find_case_insensitive_maps_expanded_unicode_lowercase_to_original_columns() {
    let buffer = TextBuffer::from_text("aßb SS\nİstanbul\n");
    let insensitive = SearchMode {
        case_sensitive: false,
    };

    assert_eq!(
        buffer.find_next_with_mode("ss", Cursor { row: 0, column: 0 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_next_with_mode("i", Cursor { row: 1, column: 0 }, insensitive),
        Some(Cursor { row: 1, column: 0 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("ss", Cursor { row: 0, column: 6 }, insensitive),
        Some(Cursor { row: 0, column: 4 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("ss", Cursor { row: 0, column: 3 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
}

#[test]
fn find_case_insensitive_handles_partial_folded_graphemes_without_duplicates() {
    let buffer = TextBuffer::from_text("aßb\nİx\n");
    let insensitive = SearchMode {
        case_sensitive: false,
    };

    assert_eq!(
        buffer.find_next_with_mode("s", Cursor { row: 0, column: 0 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_previous_with_mode("s", Cursor { row: 0, column: 3 }, insensitive),
        Some(Cursor { row: 0, column: 1 })
    );
    assert_eq!(
        buffer.find_next_with_mode("\u{307}", Cursor { row: 1, column: 0 }, insensitive),
        Some(Cursor { row: 1, column: 0 })
    );
}

#[test]
fn find_previous_finds_query_before_cursor() {
    let buffer = TextBuffer::from_text("alpha\nbeta alpha\n");

    assert_eq!(
        buffer.find_previous("alpha", Cursor { row: 1, column: 10 }),
        Some(Cursor { row: 1, column: 5 })
    );
}

#[test]
fn find_previous_wraps_to_bottom() {
    let buffer = TextBuffer::from_text("first\nsecond match\n");

    assert_eq!(
        buffer.find_previous("match", Cursor { row: 0, column: 0 }),
        Some(Cursor { row: 1, column: 7 })
    );
}

#[test]
fn find_previous_handles_unicode_columns() {
    let buffer = TextBuffer::from_text("aé match\n");

    assert_eq!(
        buffer.find_previous("match", Cursor { row: 0, column: 8 }),
        Some(Cursor { row: 0, column: 3 })
    );
}

#[test]
fn repeat_search_with_mode_wraps_and_honors_case() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("Alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 1, column: 4 };

    assert_eq!(
        repeat_search_next_with_mode(
            &document,
            &mut cursor,
            "alpha",
            SearchMode {
                case_sensitive: false,
            },
        ),
        SearchRepeatResult::Found {
            query: "alpha".to_string()
        }
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}
