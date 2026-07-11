use super::*;
use crate::tui::input::*;

#[test]
fn ctrl_z_undo_restores_buffer_and_clamps_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
    };
    document
        .buffer
        .delete_char(0, 5)
        .expect("join lines for setup");
    let mut cursor = Cursor { row: 1, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL)
    ));

    assert_eq!(
        document.buffer.lines(),
        &["hello".to_string(), "world".to_string()]
    );
    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Undone");
}

#[test]
fn ctrl_y_redo_restores_buffer_and_clamps_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\nworld\n"),
    };
    document
        .buffer
        .delete_char(0, 5)
        .expect("join lines for setup");
    assert!(document.buffer.undo_last_edit());
    let mut cursor = Cursor { row: 1, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.lines(), &["helloworld".to_string()]);
    assert_eq!(cursor, Cursor { row: 0, column: 10 });
    assert_eq!(runtime.status, "Redone");
}

#[test]
fn f3_repeats_last_search() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
    ));
    for key in [
        KeyCode::Char('a'),
        KeyCode::Char('l'),
        KeyCode::Char('p'),
        KeyCode::Char('h'),
        KeyCode::Char('a'),
        KeyCode::Enter,
    ] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.last_search_query, "alpha");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f3_repeats_last_search_case_insensitive_by_default() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("Alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 1, column: 4 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn f3_without_search_reports_missing_query() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "No previous search");
}

#[test]
fn shift_f3_repeats_last_search_backwards() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta alpha\ngamma alpha\n"),
    };
    let mut cursor = Cursor { row: 2, column: 7 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 5 });
    assert_eq!(runtime.status, "Found: alpha");
}

#[test]
fn shift_f3_without_search_reports_missing_query() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::SHIFT)
    ));

    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "No previous search");
}

#[test]
fn ctrl_g_goes_to_line_and_clamps_column() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 0, column: 99 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.goto_line_active);
    assert_eq!(runtime.status, "Go to line: ");

    for key in [KeyCode::Char('3'), KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert!(!runtime.goto_line_active);
    assert_eq!(cursor, Cursor { row: 2, column: 5 });
    assert_eq!(runtime.status, "Line 3");
}

#[test]
fn go_to_line_rejects_out_of_range_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
    };
    let mut cursor = Cursor { row: 1, column: 1 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)
    ));
    for key in [KeyCode::Char('9'), KeyCode::Char('9'), KeyCode::Enter] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert!(!runtime.goto_line_active);
    assert_eq!(cursor, Cursor { row: 1, column: 1 });
    assert_eq!(runtime.status, "Line out of range: 99");
}

#[test]
fn ctrl_f_search_moves_cursor_to_match() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.search_active);

    for value in ['b', 'e', 't', 'a'] {
        assert!(!handle_key_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE)
        ));
    }

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));

    assert_eq!(cursor, Cursor { row: 1, column: 0 });
    assert!(!runtime.search_active);
    assert_eq!(runtime.status, "Found: beta");
}

#[test]
fn ctrl_f_search_remembers_history_and_recalls_with_arrows() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\ngamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    for query in ["alpha", "beta", "alpha"] {
        start_search(&mut runtime);
        runtime.search_query = query.to_string();
        handle_search_key_event(
            &document,
            &mut cursor,
            &mut runtime,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
    }

    assert_eq!(
        runtime.search_history,
        vec![String::from("alpha"), String::from("beta")]
    );

    start_search(&mut runtime);
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "alpha");
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "beta");
    handle_search_key_event(
        &document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    assert_eq!(runtime.search_query, "alpha");
}

#[test]
fn search_case_toggle_persists_and_exact_case_changes_results() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("Alpha\nalpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        last_search_query: String::from("alpha"),
        ..EditorRuntime::default()
    };

    toggle_search_case(&mut runtime);
    assert!(runtime.settings.search_case_sensitive);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 0 });
}
