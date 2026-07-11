use super::*;
use crate::tui::input::*;

#[test]
fn save_failure_status_does_not_include_buffer_contents() {
    let secret = "SUPER_SECRET_TOKEN";
    let mut document = TextDocument {
        path: PathBuf::from("."),
        buffer: kfnotepad::TextBuffer::from_text(secret),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
    ));

    assert!(runtime.status.starts_with("Save failed:"));
    assert!(!runtime.status.contains(secret));
}

#[test]
fn tui_save_refuses_external_modification_since_open() {
    let temp = TempArea::new("tui-save-conflict");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut document = open_text_file(&path).expect("open document");
    document.buffer.insert_char(0, 0, '!').expect("edit buffer");
    fs::write(&path, "external\n").expect("external edit");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
    ));

    assert_eq!(fs::read_to_string(&path).expect("read file"), "external\n");
    assert!(document.buffer.is_dirty());
    assert!(runtime
        .status
        .contains("file changed on disk since open or last save"));
}
