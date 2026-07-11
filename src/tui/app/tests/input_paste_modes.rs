use super::*;
use crate::tui::input::*;

#[test]
fn paste_text_inserts_multiple_characters() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "ab\ncd");

    assert_eq!(document.buffer.lines(), vec!["ab", "cdhello"]);
    assert_eq!(cursor, Cursor { row: 1, column: 2 });
    assert_eq!(runtime.status, "Modified");
    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.to_text(), "hello");
    assert!(!document.buffer.undo_last_edit());
}

#[test]
fn paste_text_normalizes_crlf_sequences() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "x\r\ny");

    assert_eq!(document.buffer.lines(), vec!["x", "yhello"]);
    assert_eq!(cursor, Cursor { row: 1, column: 1 });
}

#[test]
fn paste_text_advances_cursor_to_combining_grapheme_end() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("e"),
    };
    let mut cursor = Cursor { row: 0, column: 1 };
    let mut runtime = EditorRuntime::default();

    insert_paste(&mut document, &mut cursor, &mut runtime, "\u{301}");

    assert_eq!(document.buffer.to_text(), "e\u{301}");
    assert_eq!(cursor, Cursor { row: 0, column: 2 });
}

#[test]
fn paste_adds_to_search_query_when_search_is_active() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        search_active: true,
        ..EditorRuntime::default()
    };

    insert_paste(&mut document, &mut cursor, &mut runtime, "term");

    assert_eq!(runtime.search_query, "term");
    assert_eq!(runtime.status, "Search: term");
}
