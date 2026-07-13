use super::*;
use crate::tui::input::*;
use crate::tui::render::clamp_horizontal_viewport;

#[test]
fn horizontal_viewport_follows_cursor_left_and_right() {
    let settings = EditorSettings::default();
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n界xyz\n"),
    };
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 5 }, settings, 4, 10, 0),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 2 }, settings, 4, 10, 4),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 0, column: 3 }, settings, 4, 10, 2),
        2
    );
    assert_eq!(
        clamp_horizontal_viewport(&document, Cursor { row: 1, column: 4 }, settings, 4, 10, 0),
        2
    );
}

#[test]
fn ctrl_w_toggles_wrap_mode() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!runtime.settings.wrap_lines);
    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
    ));
    assert!(runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap on");

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL)
    ));
    assert!(!runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap off");
}
