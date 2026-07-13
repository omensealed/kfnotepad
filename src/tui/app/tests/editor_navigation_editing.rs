use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;

#[test]
fn page_up_and_down_move_by_visible_page() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\nfive\n"),
    };
    let mut cursor = Cursor { row: 1, column: 3 };
    let mut runtime = EditorRuntime {
        page_rows: 2,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 3, column: 3 });
    assert_eq!(runtime.status, "Page down");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 3 });
    assert_eq!(runtime.status, "Page up");

    cursor = Cursor { row: 3, column: 4 };
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 4, column: 4 });
}

#[test]
#[cfg(feature = "syntax")]
fn syntax_highlighter_detects_rust_and_falls_back_to_plain_text() {
    let highlighter = SyntaxHighlighter::default();
    let rust_document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("fn main() {}\n"),
    };
    let text_document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("plain note\n"),
    };

    assert_eq!(highlighter.syntax_name_for_document(&rust_document), "Rust");
    assert_eq!(
        highlighter.syntax_name_for_document(&text_document),
        "Plain Text"
    );
    assert!(highlighter
        .highlight_line(&rust_document, "fn main() {}")
        .is_some());
    assert!(highlighter
        .highlight_line(&text_document, "plain note")
        .is_none());
}

#[test]
#[cfg(feature = "syntax")]
fn syntax_highlighter_keeps_state_before_viewport() {
    let highlighter = SyntaxHighlighter::default();
    let document = TextDocument {
        path: PathBuf::from("main.rs"),
        buffer: kfnotepad::TextBuffer::from_text("/* start\ninside\n*/\nfn main() {}\n"),
    };

    let stateful = highlighter.highlight_visible_lines(&document, 1, 1);
    let reset = highlighter
        .highlight_line(&document, "inside")
        .expect("standalone Rust line highlights");
    let stateful_line = stateful
        .first()
        .and_then(Option::as_ref)
        .expect("stateful Rust line highlights");

    assert_ne!(stateful_line[0].0.foreground, reset[0].0.foreground);
    assert_eq!(stateful_line[0].1, "inside");
}

#[test]
fn viewport_follows_cursor_down_and_up() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\n"),
    };

    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 3, column: 0 },
            0,
            2,
            EditorSettings::default(),
            2,
            80
        ),
        2
    );
    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 0, column: 0 },
            2,
            2,
            EditorSettings::default(),
            2,
            80
        ),
        0
    );
}

#[test]
fn wrapped_viewport_can_scroll_to_last_source_line() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one two three four five\nlast\n"),
    };
    let settings = EditorSettings {
        wrap_lines: true,
        ..EditorSettings::default()
    };

    assert_eq!(
            clamp_passive_viewport(&document, 99, 10, settings),
            1,
            "wrapped mode must allow the final source line to become visible even when source line count is smaller than visible rows"
        );
    assert_eq!(
        clamp_viewport(
            &document,
            Cursor { row: 1, column: 0 },
            0,
            3,
            settings,
            2,
            10
        ),
        1,
        "cursor-following clamp must account for visual rows in wrapped mode"
    );
}

#[test]
fn dirty_quit_requires_second_ctrl_q() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();
    let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        quit
    ));
    assert!(runtime.quit_confirmation_pending);
    assert!(runtime.status.contains("Unsaved changes"));

    assert!(handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        quit
    ));
}

#[test]
fn backspace_key_updates_buffer_and_cursor() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 5 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)
    ));

    assert_eq!(document.buffer.line(0), Some("hell"));
    assert_eq!(cursor, Cursor { row: 0, column: 4 });
    assert!(document.buffer.is_dirty());
}

#[test]
fn ctrl_backspace_and_ctrl_delete_update_buffer_by_word() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 11 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Backspace, KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.line(0), Some("alpha gamma"));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Delete, KeyModifiers::CONTROL)
    ));

    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn ctrl_k_and_edit_menu_delete_to_line_end() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha beta gamma\nnext\n"),
    };
    let mut cursor = Cursor { row: 0, column: 6 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)
    ));
    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
    assert!(!runtime.quit_confirmation_pending);

    assert!(document.buffer.undo_last_edit());
    cursor = Cursor { row: 0, column: 6 };
    runtime.menu = Some(MenuState {
        group: MenuGroup::Edit,
        selected: 6,
    });
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));

    assert_eq!(runtime.menu, None);
    assert_eq!(document.buffer.line(0), Some("alpha "));
    assert_eq!(cursor, Cursor { row: 0, column: 6 });
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn home_and_end_keys_move_within_current_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
    };
    let mut cursor = Cursor { row: 0, column: 2 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::End, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}

#[test]
fn insert_key_toggles_overwrite_mode_for_typed_characters() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcd\n"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
    ));
    assert!(runtime.overwrite_mode);
    assert_eq!(runtime.status, "Overwrite on");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("aXcd"));
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
    assert_eq!(runtime.status, "Modified overwrite");

    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.line(0), Some("abcd"));
    assert!(!document.buffer.undo_last_edit());

    cursor.column = 4;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('!'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("abcd!"));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Insert, KeyModifiers::NONE)
    ));
    assert!(!runtime.overwrite_mode);
    assert_eq!(runtime.status, "Insert mode");
    cursor.column = 1;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT)
    ));
    assert_eq!(document.buffer.line(0), Some("aYbcd!"));
    assert_eq!(runtime.status, "Modified");
}

#[test]
fn overwrite_replaces_whole_grapheme_cluster_and_advances_to_boundary() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("a🇺🇸e\u{301}!"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime {
        overwrite_mode: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("axe\u{301}!"));
    assert_eq!(cursor, Cursor { row: 0, column: 2 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("axy!"));
    assert_eq!(cursor, Cursor { row: 0, column: 3 });
}

#[test]
fn ctrl_a_and_ctrl_e_move_within_current_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("héllo\n"),
    };
    let mut cursor = Cursor { row: 0, column: 2 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 5 });
    assert!(!runtime.quit_confirmation_pending);

    runtime.quit_confirmation_pending = true;
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert!(!runtime.quit_confirmation_pending);
}

#[test]
fn ctrl_home_and_end_move_to_document_edges() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut cursor = Cursor { row: 1, column: 2 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 2, column: 5 });
    assert_eq!(runtime.status, "Bottom");
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.status, "Top");
}

#[test]
fn ctrl_left_and_right_move_by_words() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha, beta\n  gamma\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 7 });
    assert!(!runtime.quit_confirmation_pending);

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 1, column: 2 });

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)
    ));
    assert_eq!(cursor, Cursor { row: 0, column: 7 });
}

#[test]
fn tab_and_backtab_indent_and_unindent_current_line() {
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
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("    alpha"));
    assert_eq!(cursor.column, 4);
    assert_eq!(runtime.status, "Indented");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.line(0), Some("alpha"));
    assert_eq!(cursor.column, 0);
    assert_eq!(runtime.status, "Unindented");
}
