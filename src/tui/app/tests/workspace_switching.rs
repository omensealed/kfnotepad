use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;

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
fn editor_tab_state_keeps_cursor_and_viewport_independent() {
    let first = EditorTabState {
        cursor: Cursor { row: 3, column: 7 },
        viewport_start: 2,
        horizontal_offset: 5,
    };
    let second = EditorTabState::default();

    assert_eq!(first.cursor, Cursor { row: 3, column: 7 });
    assert_eq!(first.viewport_start, 2);
    assert_eq!(first.horizontal_offset, 5);
    assert_eq!(second.cursor, Cursor { row: 0, column: 0 });
    assert_eq!(second.viewport_start, 0);
    assert_eq!(second.horizontal_offset, 0);
}

#[test]
fn editor_workspace_starts_with_one_active_tab() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let workspace = EditorWorkspace::from_document(&mut document);

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
    assert_eq!(
        workspace.active_tab().document.as_ref().path,
        PathBuf::from("note.txt")
    );
    assert_eq!(workspace.active_tab().state, EditorTabState::default());
}

#[test]
fn editor_workspace_active_tab_mutates_original_document() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
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
            .expect("insert into active tab");

        assert_eq!(active_tab.state.cursor, Cursor { row: 0, column: 5 });
        assert!(active_tab.document.as_ref().buffer.is_dirty());
    }

    assert_eq!(document.buffer.to_text(), "alpha!\n");
    assert!(document.buffer.is_dirty());
}

#[test]
fn workspace_tab_switch_reports_single_tab_without_moving() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    select_next_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.active, 0);
    assert_eq!(runtime.status, "Only one tab open");
    assert!(!runtime.quit_confirmation_pending);

    select_previous_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.active, 0);
    assert_eq!(runtime.status, "Only one tab open");
}

#[test]
fn workspace_tab_switch_cycles_between_tabs() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: kfnotepad::TextBuffer::from_text("two\n"),
    };
    let mut workspace = EditorWorkspace {
        tabs: vec![
            EditorTab {
                document: EditorTabDocument::Borrowed(&mut first),
                state: EditorTabState {
                    cursor: Cursor { row: 0, column: 1 },
                    viewport_start: 0,
                    horizontal_offset: 0,
                },
            },
            EditorTab {
                document: EditorTabDocument::Borrowed(&mut second),
                state: EditorTabState {
                    cursor: Cursor { row: 0, column: 2 },
                    viewport_start: 0,
                    horizontal_offset: 0,
                },
            },
        ],
        active: 0,
    };
    let mut runtime = EditorRuntime::default();

    select_next_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.active, 1);
    assert_eq!(
        workspace.active_tab().state.cursor,
        Cursor { row: 0, column: 2 }
    );
    assert_eq!(runtime.status, "Tab 2/2: second.txt");

    select_next_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.active, 0);
    assert_eq!(
        workspace.active_tab().state.cursor,
        Cursor { row: 0, column: 1 }
    );
    assert_eq!(runtime.status, "Tab 1/2: first.txt");

    select_previous_tab(&mut workspace, &mut runtime);
    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Tab 2/2: second.txt");
}

#[test]
fn workspace_tab_keybindings_switch_only_when_editor_body_is_active() {
    let mut first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: kfnotepad::TextBuffer::from_text("two\n"),
    };
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
        active: 0,
    };
    let next_tab = KeyEvent::new(KeyCode::PageDown, KeyModifiers::CONTROL);
    let mut runtime = EditorRuntime {
        menu: Some(MenuState::default()),
        ..EditorRuntime::default()
    };

    assert!(!handle_workspace_key_event(
        &mut workspace,
        &mut runtime,
        next_tab
    ));
    assert_eq!(workspace.active, 0);

    runtime.menu = None;
    assert!(handle_workspace_key_event(
        &mut workspace,
        &mut runtime,
        next_tab
    ));
    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Tab 2/2: second.txt");
}
