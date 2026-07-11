use super::*;
use crate::tui::input::*;

#[test]
fn ctrl_q_works_while_sidebar_is_open() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState {
            current_dir: PathBuf::from("."),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
        }),
        ..EditorRuntime::default()
    };

    assert!(handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)
    ));
}

#[test]
fn dirty_ctrl_q_confirmation_works_while_sidebar_is_open() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty document");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState {
            current_dir: PathBuf::from("."),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
        }),
        ..EditorRuntime::default()
    };
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
fn dirty_ctrl_c_confirmation_works_like_ctrl_q() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty document");
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();
    let quit = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

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
fn workspace_close_dirty_tab_requires_confirmation() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: kfnotepad::TextBuffer::from_text("two\n"),
    };
    second
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty second tab");
    let mut workspace = EditorWorkspace {
        tabs: vec![
            EditorTab {
                document: EditorTabDocument::Borrowed(&mut first),
                state: EditorTabState::default(),
            },
            EditorTab {
                document: EditorTabDocument::Borrowed(&mut second),
                state: EditorTabState::default(),
            },
        ],
        active: 1,
    };
    let mut runtime = EditorRuntime::default();

    close_active_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert!(runtime.close_tab_confirmation_pending);
    assert_eq!(
        runtime.status,
        "Unsaved changes. Press Ctrl-F4 again to close tab."
    );

    close_active_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
    assert!(!runtime.close_tab_confirmation_pending);
    assert_eq!(runtime.status, "Closed tab: second.txt");
}
