use super::*;

#[test]
fn editor_tab_state_starts_at_document_origin() {
    assert_eq!(
        EditorTabState::default(),
        EditorTabState {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
        }
    );
}

#[test]
fn editor_workspace_keeps_borrowed_document_editable() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha\n"),
    };

    {
        let mut workspace = EditorWorkspace::from_document(&mut document);
        let active_tab = workspace.active_tab_mut();
        active_tab.state.cursor = Cursor { row: 0, column: 5 };
        active_tab
            .document
            .as_mut()
            .buffer
            .insert_char(0, 5, '!')
            .expect("insert through active tab");

        assert_eq!(active_tab.state.cursor, Cursor { row: 0, column: 5 });
        assert!(active_tab.document.as_ref().buffer.is_dirty());
    }

    assert_eq!(document.buffer.to_text(), "alpha!\n");
    assert!(document.buffer.is_dirty());
}

#[test]
fn editor_workspace_switches_and_closes_tabs_without_terminal_types() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut first);

    assert!(!workspace.select_next_tab());
    workspace.push_owned_tab(second);
    assert_eq!(workspace.active, 1);
    assert_eq!(
        workspace.tab_strip_items(),
        vec![
            TabStripItem {
                label: String::from("first.txt"),
                active: false,
                dirty: false,
            },
            TabStripItem {
                label: String::from("second.txt"),
                active: true,
                dirty: false,
            },
        ]
    );

    assert!(workspace.select_next_tab());
    assert_eq!(workspace.active, 0);
    assert!(workspace.select_previous_tab());
    assert_eq!(workspace.active, 1);

    assert_eq!(
        workspace.close_active_tab(false),
        CloseActiveTabResult::Closed {
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
}

#[test]
fn editor_workspace_requires_confirmation_before_dirty_tab_close() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    second
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty second tab");
    let mut workspace = EditorWorkspace::from_document(&mut first);
    workspace.push_owned_tab(second);

    assert_eq!(
        workspace.close_active_tab(false),
        CloseActiveTabResult::Dirty
    );
    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(
        workspace.close_active_tab(true),
        CloseActiveTabResult::Closed {
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tabs.len(), 1);
}

#[test]
fn shared_document_commands_edit_and_clamp_cursor_without_terminal_types() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha beta\nsecond\n"),
    };
    let mut cursor = Cursor { row: 0, column: 10 };

    assert_eq!(
        delete_previous_word(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        undo_document_edit(&mut document, &mut cursor),
        UndoRedoResult::Applied
    );
    assert_eq!(document.buffer.to_text(), "alpha beta\nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        redo_document_edit(&mut document, &mut cursor),
        UndoRedoResult::Applied
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    assert_eq!(
        delete_to_line_end(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), "alpha \nsecond\n");
    cursor.column = 0;
    assert_eq!(
        delete_next_word(&mut document, &mut cursor),
        EditResult::Modified
    );
    assert_eq!(document.buffer.to_text(), " \nsecond\n");
}

#[test]
fn shared_navigation_commands_move_without_terminal_types() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha beta\nsecond line\nthird\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };

    move_document_cursor(&document, &mut cursor, CursorMove::WordRight);
    assert_eq!(cursor, Cursor { row: 0, column: 6 });

    page_down(&document, &mut cursor, 2);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });

    go_to_document_end(&document, &mut cursor);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });

    page_up(&document, &mut cursor, 10);
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    go_to_document_start(&mut cursor);
    assert_eq!(cursor, Cursor { row: 0, column: 0 });

    cursor = Cursor { row: 0, column: 99 };
    assert_eq!(
        go_to_line(&document, &mut cursor, "2"),
        GoToLineResult::Moved { line_number: 2 }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 11 });
    assert_eq!(
        go_to_line(&document, &mut cursor, ""),
        GoToLineResult::Empty
    );
    assert_eq!(
        go_to_line(&document, &mut cursor, "abc"),
        GoToLineResult::Invalid
    );
    assert_eq!(
        go_to_line(&document, &mut cursor, "99"),
        GoToLineResult::OutOfRange { line_number: 99 }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 11 });
}

#[test]
fn shared_repeat_search_updates_cursor_and_reports_result() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };

    assert_eq!(
        repeat_search_next(&document, &mut cursor, ""),
        SearchRepeatResult::NoPreviousSearch
    );
    assert_eq!(
        repeat_search_next(&document, &mut cursor, "alpha"),
        SearchRepeatResult::Found {
            query: String::from("alpha"),
        }
    );
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(
        repeat_search_previous(&document, &mut cursor, "alpha"),
        SearchRepeatResult::Found {
            query: String::from("alpha"),
        }
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(
        repeat_search_next(&document, &mut cursor, "missing"),
        SearchRepeatResult::NoMatch {
            query: String::from("missing"),
        }
    );
}
