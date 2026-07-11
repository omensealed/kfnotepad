use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;
use crate::tui::render::*;

#[test]
fn file_sidebar_delete_refuses_open_dirty_file_and_symlink() {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("sidebar-delete-refuse");
    let dirty_path = temp.path("dirty.txt");
    let target_path = temp.path("target.txt");
    #[cfg(unix)]
    let link_path = temp.path("link.txt");
    fs::write(&dirty_path, "dirty\n").expect("write dirty file");
    fs::write(&target_path, "target\n").expect("write target file");
    #[cfg(unix)]
    symlink(&target_path, &link_path).expect("create symlink");

    let mut document = TextDocument {
        path: dirty_path.clone(),
        buffer: kfnotepad::TextBuffer::from_text("dirty\n"),
    };
    document.buffer.insert_char(0, 0, '!').expect("mark dirty");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState {
            current_dir: temp.root.clone(),
            entries: vec![FileSidebarEntry {
                label: String::from("dirty.txt"),
                path: dirty_path.clone(),
                kind: FileSidebarEntryKind::File,
            }],
            selected: 0,
            scroll: 0,
        }),
        sidebar_prompt: Some(SidebarPrompt::DeleteConfirm {
            entry: FileSidebarEntry {
                label: String::from("dirty.txt"),
                path: dirty_path.clone(),
                kind: FileSidebarEntryKind::File,
            },
            recursive: false,
        }),
        sidebar_query: String::from("yes"),
        ..EditorRuntime::default()
    };

    apply_sidebar_prompt(&mut workspace, &mut runtime);
    assert!(dirty_path.exists());
    assert_eq!(runtime.status, "Cannot delete an open modified file");

    #[cfg(unix)]
    {
        runtime.sidebar_prompt = Some(SidebarPrompt::DeleteConfirm {
            entry: FileSidebarEntry {
                label: String::from("link.txt"),
                path: link_path.clone(),
                kind: FileSidebarEntryKind::File,
            },
            recursive: false,
        });
        runtime.sidebar_query = String::from("yes");
        apply_sidebar_prompt(&mut workspace, &mut runtime);
        assert!(link_path.exists());
        assert_eq!(runtime.status, "Refusing to delete symlink");
    }
}

#[test]
fn mouse_click_on_tab_strip_switches_workspace_tab() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: kfnotepad::TextBuffer::from_text("first\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: kfnotepad::TextBuffer::from_text("second\n"),
    };
    let mut workspace = EditorWorkspace {
        tabs: vec![
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(first)),
                state: EditorTabState::default(),
            },
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(second)),
                state: EditorTabState::default(),
            },
        ],
        active: 0,
    };
    let mut runtime = EditorRuntime::default();
    let first_label_width = text_display_width(" 1:first.txt ");

    assert_eq!(
        handle_workspace_mouse_event(
            &mut workspace,
            &mut runtime,
            left_click(first_label_width as u16 + 1, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 2,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Tab 2/2: second.txt");
}

#[test]
fn sidebar_mouse_wheel_moves_selection_without_wrapping() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        sidebar: Some(sidebar_fixture(12)),
        ..EditorRuntime::default()
    };
    let context = MouseContext {
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        gutter_width: 4,
        terminal_width: 80,
        sidebar_width: SIDEBAR_WIDTH,
        body_top: 1,
    };

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollDown, 2, 2),
            context
        ),
        InputResult::Handled
    );
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 1);
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);

    for _ in 0..3 {
        assert_eq!(
            handle_mouse_event(
                &mut document,
                &mut cursor,
                &mut runtime,
                mouse_event(MouseEventKind::ScrollDown, 2, 2),
                context
            ),
            InputResult::Handled
        );
    }
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 4);
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 2);

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollUp, 2, 2),
            context
        ),
        InputResult::Handled
    );
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 3);
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 2);

    runtime.sidebar.as_mut().expect("sidebar").selected = 0;
    runtime.sidebar.as_mut().expect("sidebar").scroll = 0;
    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollUp, 2, 2),
            context
        ),
        InputResult::Ignored
    );
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 0);
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);
}

#[test]
fn editor_body_mouse_wheel_moves_cursor_by_rows() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\nfive\nsix\nseven\n"),
    };
    let mut cursor = Cursor { row: 1, column: 2 };
    let mut runtime = EditorRuntime {
        sidebar: Some(sidebar_fixture(4)),
        ..EditorRuntime::default()
    };

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollDown, (SIDEBAR_WIDTH + 2) as u16, 2),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 3,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: SIDEBAR_WIDTH,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );
    assert_eq!(cursor, Cursor { row: 4, column: 2 });
    assert_eq!(runtime.status, "Scroll down");
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").selected, 0);
    assert_eq!(runtime.sidebar.as_ref().expect("sidebar").scroll, 0);

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollUp, (SIDEBAR_WIDTH + 2) as u16, 2),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 3,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: SIDEBAR_WIDTH,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );
    assert_eq!(cursor, Cursor { row: 1, column: 2 });
    assert_eq!(runtime.status, "Scroll up");
}

#[test]
fn editor_body_mouse_wheel_ignores_header_and_active_menu() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\nfour\n"),
    };
    let mut cursor = Cursor { row: 1, column: 0 };
    let mut runtime = EditorRuntime::default();
    let context = MouseContext {
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        gutter_width: 4,
        terminal_width: 80,
        sidebar_width: 0,
        body_top: 1,
    };

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollDown, 5, 0),
            context
        ),
        InputResult::Ignored
    );
    assert_eq!(cursor, Cursor { row: 1, column: 0 });

    runtime.menu = Some(MenuState {
        group: MenuGroup::File,
        selected: 0,
    });
    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            mouse_event(MouseEventKind::ScrollDown, 5, 2),
            context
        ),
        InputResult::Ignored
    );
    assert_eq!(cursor, Cursor { row: 1, column: 0 });
}

fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}
