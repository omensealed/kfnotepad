use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;

#[test]
fn workspace_close_tab_refuses_only_tab() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        close_tab_confirmation_pending: true,
        quit_confirmation_pending: true,
        ..EditorRuntime::default()
    };

    close_active_tab(&mut workspace, &mut runtime);

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
    assert_eq!(runtime.status, "Cannot close the only tab");
    assert!(!runtime.close_tab_confirmation_pending);
    assert!(!runtime.quit_confirmation_pending);
}

#[test]
fn workspace_close_tab_removes_clean_active_tab_and_clamps_selection() {
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
                state: EditorTabState {
                    cursor: Cursor { row: 0, column: 2 },
                    viewport_start: 0,
                    horizontal_offset: 0,
                },
            },
        ],
        active: 1,
    };
    let mut runtime = EditorRuntime::default();

    close_active_tab(&mut workspace, &mut runtime);

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active, 0);
    assert_eq!(
        workspace.active_tab().document.as_ref().path,
        PathBuf::from("first.txt")
    );
    assert_eq!(runtime.status, "Closed tab: second.txt");
    assert!(!runtime.close_tab_confirmation_pending);
}

#[test]
fn workspace_close_tab_keybinding_works_only_when_editor_body_is_active() {
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
        active: 1,
    };
    let close_tab = KeyEvent::new(KeyCode::F(4), KeyModifiers::CONTROL);
    let mut runtime = EditorRuntime {
        search_active: true,
        ..EditorRuntime::default()
    };

    assert!(!handle_workspace_key_event(
        &mut workspace,
        &mut runtime,
        close_tab
    ));
    assert_eq!(workspace.tabs.len(), 2);

    runtime.search_active = false;
    assert!(handle_workspace_key_event(
        &mut workspace,
        &mut runtime,
        close_tab
    ));
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(runtime.status, "Closed tab: second.txt");
}

#[test]
fn f10_tabs_menu_switches_and_closes_tabs() {
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
    let mut runtime = EditorRuntime {
        menu: Some(MenuState {
            group: MenuGroup::Tabs,
            selected: 1,
        }),
        ..EditorRuntime::default()
    };

    assert!(!handle_workspace_menu_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Tab 2/2: second.txt");

    runtime.menu = Some(MenuState {
        group: MenuGroup::Tabs,
        selected: 2,
    });
    assert!(!handle_workspace_menu_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(runtime.status, "Closed tab: second.txt");
}
