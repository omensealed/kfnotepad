#[test]
fn mouse_click_moves_cursor_in_editor_body() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\nbeta\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(8, 2),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 1, column: 2 });
    assert!(!runtime.quit_confirmation_pending);
}

#[test]
fn mouse_click_respects_horizontal_offset() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(5, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 3,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 3 });
}

#[test]
fn mouse_click_respects_reserved_sidebar_width() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdef\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click((SIDEBAR_WIDTH + 8) as u16, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: SIDEBAR_WIDTH,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 2 });
}

#[test]
fn mouse_click_on_wrapped_visual_row_renders_and_edits_same_line() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdefghij\nsecond\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        settings: EditorSettings {
            wrap_lines: true,
            ..EditorSettings::default()
        },
        ..EditorRuntime::default()
    };
    let context = MouseContext {
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        gutter_width: 2,
        terminal_width: 10,
        sidebar_width: 0,
        body_top: 1,
    };

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(6, 2),
            context
        ),
        InputResult::Handled
    );

    assert_eq!(cursor, Cursor { row: 0, column: 8 });

    let frame = RenderFrame {
        theme: EditorTheme::for_id(runtime.settings.theme_id),
        gutter_width: context.gutter_width,
        terminal_width: context.terminal_width,
        origin_column: 0,
        body_top: context.body_top,
        no_color: false,
    };
    let view = EditorView {
        cursor,
        viewport_start: 0,
        horizontal_offset: 0,
        visible_rows: 3,
        status: "",
        settings: runtime.settings,
        menu: None,
        sidebar_width: 0,
        tab_strip: &[],
        search_highlight: None,
    };

    assert_eq!(cursor_screen_row(&document, view, frame), 2);
    assert_eq!(cursor_screen_column(&document, cursor, view, frame), 6);
    assert!(cursor_row_is_visible(&document, view, frame));

    assert!(!handle_key_event(
        &mut document,
        &mut cursor,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE)
    ));
    assert_eq!(document.buffer.lines()[0], "abcdefghXij");
    assert_eq!(document.buffer.lines()[1], "second");
}

#[test]
fn mouse_click_opens_menu_group_and_runs_dropdown_item() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(18, 0),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );
    assert_eq!(
        runtime.menu,
        Some(MenuState {
            group: MenuGroup::Edit,
            selected: 0
        })
    );

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            left_click(19, 1),
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Handled
    );

    assert_eq!(runtime.menu, None);
    assert!(runtime.search_active);
    assert_eq!(runtime.status, "Search: ");
}

#[test]
fn mouse_move_is_ignored_without_requesting_redraw() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("alpha\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime::default();

    assert_eq!(
        handle_mouse_event(
            &mut document,
            &mut cursor,
            &mut runtime,
            MouseEvent {
                kind: MouseEventKind::Moved,
                column: 10,
                row: 1,
                modifiers: KeyModifiers::NONE,
            },
            MouseContext {
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 10,
                gutter_width: 4,
                terminal_width: 80,
                sidebar_width: 0,
                body_top: 1,
            }
        ),
        InputResult::Ignored
    );
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
}

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
