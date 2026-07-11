#[test]
fn file_sidebar_delete_refuses_open_dirty_file_and_symlink() {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    let temp = TempArea::new("sidebar-delete-refuse");
    let dirty_path = temp.path("dirty.txt");
    let target_path = temp.path("target.txt");
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

fn left_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn sidebar_fixture(count: usize) -> FileSidebarState {
    FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..count)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 0,
    }
}

#[test]
fn render_marks_dirty_buffer_and_controls() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    document.buffer.insert_char(0, 5, '!').expect("edit buffer");
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 6 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 20,
            status: "Ctrl-S save | Ctrl-Q quit",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("kfnotepad"));
    assert!(output.contains("note.txt"));
    assert!(output.contains("modified"));
    assert!(output.contains(" 1"));
    assert!(output.contains("hello!"));
    assert!(output.contains("Ln 1, Col 7"));
    assert!(output.contains("wrap:off"));
    assert!(output.contains("modified"));
    assert!(output.contains("F10 Menu/Help"));
}

#[test]
fn render_workspace_manager_overlay_shows_actions_and_projects() {
    let manager = WorkspaceManagerState {
        entries: vec![
            WorkspaceManagerEntry {
                name: String::from("Alpha"),
                files: 2,
            },
            WorkspaceManagerEntry {
                name: String::from("Beta"),
                files: 1,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_workspace_manager_overlay(
        &mut output,
        &manager,
        12,
        0,
        80,
        1,
        EditorTheme::for_id(EditorThemeId::Nocturne),
        false,
    )
    .expect("render manager");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Workspaces"));
    assert!(output.contains("Alpha"));
    assert!(output.contains("> Beta"));
    assert!(output.contains("D delete"));
    assert!(output.contains("+ Workspaces "));
    assert!(output.contains("|"));
}

#[test]
fn render_workspace_manager_overlay_without_colors_skips_color_escapes() {
    let manager = WorkspaceManagerState {
        entries: vec![
            WorkspaceManagerEntry {
                name: String::from("Alpha"),
                files: 2,
            },
            WorkspaceManagerEntry {
                name: String::from("Beta"),
                files: 1,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_workspace_manager_overlay(
        &mut output,
        &manager,
        12,
        0,
        80,
        1,
        EditorTheme::for_id(EditorThemeId::Nocturne),
        true,
    )
    .expect("render manager");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}

#[test]
fn render_tab_strip_wraps_long_tab_labels_before_editor_body() {
    let document = TextDocument {
        path: PathBuf::from("active.txt"),
        buffer: kfnotepad::TextBuffer::from_text("body\n"),
    };
    let tabs = vec![
        TabStripItem {
            label: String::from("getting-started-guide.md"),
            active: false,
            dirty: false,
        },
        TabStripItem {
            label: String::from("release-notes-draft.md"),
            active: false,
            dirty: false,
        },
        TabStripItem {
            label: String::from("keyboard-shortcuts-reference.md"),
            active: true,
            dirty: false,
        },
    ];
    let mut output = Vec::new();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "ready",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &tabs,
            search_highlight: None,
        },
        &SyntaxHighlighter::default(),
        42,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert_eq!(tab_strip_height_for_width(&tabs, 42), 4);
    assert!(output.contains("\x1b[2;1H"));
    assert!(output.contains("\x1b[3;1H"));
    assert!(output.contains("\x1b[4;1H"));
    assert!(output.contains("\x1b[5;1H\x1b[2K"));
    assert!(output.contains("body"));
}

#[test]
fn render_does_not_clear_entire_screen_between_frames() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            status: "ready",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[2J"));
    assert!(output.contains("\x1b[2K"));
}

#[test]
fn render_clears_empty_body_rows_without_full_screen_clear() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("only line\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            status: "ready",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\x1b[2;1H\x1b[2K"));
    assert!(output.contains("\x1b[3;1H\x1b[2K"));
    assert!(output.contains("\x1b[4;1H\x1b[2K"));
    assert!(!output.contains("\x1b[2J"));
}

#[test]
fn render_tab_strip_shows_active_and_dirty_tabs_above_body() {
    let document = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: kfnotepad::TextBuffer::from_text("body\n"),
    };
    let tabs = vec![
        TabStripItem {
            label: String::from("first.txt"),
            active: false,
            dirty: true,
        },
        TabStripItem {
            label: String::from("second.txt"),
            active: true,
            dirty: false,
        },
    ];
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "ready",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &tabs,
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\x1b[2;1H\x1b[2K"));
    assert!(output.contains(" 1:first.txt* "));
    assert!(output.contains(" 2:second.txt "));
    assert!(output.contains("\x1b[3;1H\x1b[2K"));
    assert!(output.contains("body"));
    assert!(output.contains("\x1b[5;1H"));
}

#[test]
fn render_preserves_header_state_when_path_is_long() {
    let document = TextDocument {
        path: PathBuf::from("/very/long/path/that/would/otherwise/hide/the/state/note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        32,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("kfnotepad"));
    assert!(output.contains(" saved "));
    assert!(output.contains("…"));
}

#[test]
fn status_line_preserves_cursor_and_mode_metadata() {
    let status = compose_status_line(
        " very long transient status text that can be shortened ",
        " Ln 12, Col 80 | num:on | wrap:off | x:42 | nocturne | modified ",
        64,
    );

    assert_eq!(status.chars().count(), 64);
    assert!(status.contains("Col 80"));
    assert!(status.contains("wrap:off"));
    assert!(status.contains("x:42"));
    assert!(status.contains("modified"));
}

#[test]
fn search_status_preserves_query_tail_and_cursor() {
    let status = compose_prompt_status_line(
        "Search: ",
        "a very long search query",
        " Ln 1, Col 1 | num:on | wrap:off | x:0 | nocturne | saved ",
        72,
    );

    assert_eq!(status.text.chars().count(), 72);
    assert!(status.text.contains("Search:"));
    assert!(status.text.contains("…"));
    assert!(status.text.contains("ry"));
    assert!(status.text.contains("Col 1"));
    assert!(status.cursor_column.is_some());
}

#[test]
fn render_starts_at_viewport_offset() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\nthree\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 1, column: 0 },
            viewport_start: 1,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains(" 1 one"));
    assert!(output.contains(" 2"));
    assert!(output.contains("two"));
    assert!(!output.contains(" 3 three"));
}

#[test]
fn render_can_hide_line_number_gutter() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("one\ntwo\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 1 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "status",
            settings: EditorSettings {
                show_line_numbers: false,
                ..EditorSettings::default()
            },
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains(" 1 one"));
    assert!(output.contains("one"));
    assert!(output.contains("num:off"));
}

#[test]
fn render_positions_rows_and_truncates_long_lines() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\nsecond\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        10,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[2;1H"));
    assert!(output.contains("\u{1b}[3;1H"));
    assert!(output.contains("\u{1b}[4;1H"));
    assert!(output.contains(" 1 "));
    assert!(output.contains("abcde"));
    assert!(output.contains(" 2 "));
    assert!(output.contains("secon"));
    assert!(!output.contains("fgh"));
}

#[test]
fn render_reserves_columns_for_sidebar() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 10,
            tab_strip: &[],
            search_highlight: None,
        },
        &SyntaxHighlighter::default(),
        40,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[1;11H"));
    assert!(output.contains("\u{1b}[2;11H"));
    assert!(!output.contains("\u{1b}[1;1H"));
}

#[test]
fn render_keeps_terminal_cursor_visible_at_editor_cursor() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 2 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        20,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[?25h"));
    assert!(output.ends_with("\u{1b}[2;7H"));
}

#[test]
fn render_paints_active_cursor_cell() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 1 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        20,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[2;6H\u{1b}[7me\u{1b}[27m"));
    assert!(output.ends_with("\u{1b}[2;6H"));
}

#[test]
fn render_highlights_visible_search_matches() {
    assert_eq!(
        search_match_ranges(
            "Alpha beta alpha",
            "alpha",
            SearchMode {
                case_sensitive: false,
            },
        ),
        vec![0..5, 11..16]
    );
    let mut direct = Vec::new();
    let mut display_column = 0;
    let mut source_column = 0;
    let mut remaining = 10;
    assert_eq!(
        EditorTheme::default().search_bg,
        Color::Rgb {
            r: 90,
            g: 230,
            b: 245
        }
    );
    let direct_range = 0..5;
    print_line_window_with_search(
        &mut direct,
        LineWindowSearchView {
            text: "Alpha",
            start_column: 0,
            display_column: &mut display_column,
            source_column: &mut source_column,
            remaining_columns: &mut remaining,
            search_ranges: std::slice::from_ref(&direct_range),
            base_fg: None,
            frame: RenderFrame {
                theme: EditorTheme::default(),
                gutter_width: 1,
                terminal_width: 20,
                origin_column: 0,
                body_top: 1,
                no_color: false,
            },
        },
    )
    .expect("paint direct search");
    assert!(!direct.is_empty());
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("Alpha beta alpha\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 6 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            status: "ready",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: Some(SearchHighlightView {
                query: "alpha",
                mode: SearchMode {
                    case_sensitive: false,
                },
            }),
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" beta "));
}

#[test]
fn search_match_ranges_tracks_unicode_character_columns() {
    assert_eq!(
        search_match_ranges(
            "a🏳️‍🌈b",
            "🏳️‍🌈",
            SearchMode {
                case_sensitive: true,
            },
        ),
        vec![1..5]
    );
    assert_eq!(
        search_match_ranges(
            "e\u{301}x",
            "e\u{301}",
            SearchMode {
                case_sensitive: true,
            },
        ),
        vec![0..2]
    );
    assert_eq!(
        search_match_ranges(
            "🇺🇸x",
            "🇸",
            SearchMode {
                case_sensitive: true,
            },
        ),
        vec![0..2]
    );
    assert_eq!(
        search_match_ranges(
            "aßb",
            "s",
            SearchMode {
                case_sensitive: false,
            },
        ),
        vec![1..2]
    );
}

#[test]
fn render_moves_cursor_to_active_search_prompt() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "Search: beta",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Search: beta"));
    assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
    assert!(output.ends_with("\u{1b}[3;14H"));
}

#[test]
fn render_moves_cursor_to_go_to_line_prompt() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "Go to line: 42",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Go to line: 42"));
    assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
    assert!(output.ends_with("\u{1b}[3;16H"));
}

#[test]
fn render_shows_keyboard_menu_dropdown() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 2,
            status: "Menu: File",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::File,
                selected: 1,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" File "));
    assert!(output.contains(" Save"));
    assert!(output.contains("Ctrl-S"));
    assert!(output.contains(" Quit"));
    assert!(output.contains("Ctrl-Q"));
    assert!(!output.contains("\u{1b}[2;6H\u{1b}[7mh\u{1b}[27m"));
    assert!(output.contains("\u{1b}[2;12H"));
    assert!(output.ends_with("\u{1b}[3;14H"));
}

#[test]
fn render_help_menu_shows_compact_help_document_entry() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 12,
            status: "Menu: Help",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Help,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        120,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" Help "));
    assert!(output.contains("Open help document"));
    assert!(output.contains("Files and tabs"));
    assert!(output.contains("Ctrl-B / Ctrl-Enter / Ctrl-F4"));
    assert!(output.contains("Search and go"));
    assert!(output.contains("Ctrl-F / F3 / Shift-F3 / Ctrl-G"));
    assert!(output.contains("Editing"));
    assert!(output.contains("Ctrl-Z/Y / Ctrl-K / Insert"));
    assert!(output.contains("View and reader"));
    assert!(output.contains("Ctrl-L/T/R/W"));
    assert!(output.contains("Workspaces"));
    assert!(output.contains("F10 -> Workspace"));
    assert!(output.contains("Save and quit"));
    assert!(output.contains("Ctrl-S / Ctrl-Q"));
}

#[test]
fn render_anchors_edit_menu_under_header_label_without_color_spill_clear() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 4,
            status: "Menu: Edit",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Edit,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        80,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("\u{1b}[2;18H"));
    assert!(output.contains("Find"));
    assert!(output.contains("Ctrl-F"));
    assert!(output.contains("Find next"));
    assert!(output.contains("F3"));
    assert!(output.contains("Delete previous word"));
    assert!(output.contains("Ctrl-Backspace"));
    assert!(output.contains("Delete next word"));
    assert!(output.contains("Ctrl-Delete"));
    assert!(output.contains("Delete to line end"));
    assert!(output.contains("Ctrl-K"));
    assert!(!output.contains("\u{1b}[46m\u{1b}[3;18H\u{1b}[2K"));
    assert!(output.ends_with("\u{1b}[2;20H"));
}

#[test]
fn render_tabs_menu_shows_tab_commands() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 0 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 6,
            status: "Menu: Tabs",
            settings: EditorSettings::default(),
            menu: Some(MenuState {
                group: MenuGroup::Tabs,
                selected: 0,
            }),
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        100,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains(" Tabs "));
    assert!(output.contains("Previous tab"));
    assert!(output.contains("Ctrl-PageUp"));
    assert!(output.contains("Next tab"));
    assert!(output.contains("Ctrl-PageDown"));
    assert!(output.contains("Close tab"));
    assert!(output.contains("Ctrl-F4"));
    assert!(output.contains("Open sidebar file as tab"));
}

#[test]
fn help_line_uses_compact_bounded_controls() {
    let help = compose_help_line(102);

    assert_eq!(
        help.trim_end(),
        " F2 Command | F10 Menu/Help | Ctrl-S Save | Ctrl-B Files | Ctrl-Q Quit"
    );
    assert!(text_display_width(&help) <= 102);
}

#[test]
fn command_palette_filters_menu_commands_by_label_and_shortcut() {
    let wrap = command_palette_candidates("word wrap");
    assert_eq!(wrap.len(), 1);
    assert_eq!(wrap[0].command, MenuCommand::ToggleWrap);

    let save = command_palette_candidates("ctrl-s");
    assert!(save
        .iter()
        .any(|entry| entry.command == MenuCommand::Save && entry.group == MenuGroup::File));
    assert!(!save
        .iter()
        .any(|entry| entry.command == MenuCommand::HelpOnly));
}

#[test]
fn command_palette_executes_selected_workspace_command() {
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("hello\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime::default();

    open_command_palette(&mut runtime);
    for key in [
        KeyCode::Char('w'),
        KeyCode::Char('o'),
        KeyCode::Char('r'),
        KeyCode::Char('d'),
        KeyCode::Char(' '),
        KeyCode::Char('w'),
        KeyCode::Char('r'),
        KeyCode::Char('a'),
        KeyCode::Char('p'),
        KeyCode::Enter,
    ] {
        assert!(!handle_command_palette_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(key, KeyModifiers::NONE)
        ));
    }

    assert_eq!(runtime.command_palette, None);
    assert!(runtime.settings.wrap_lines);
    assert_eq!(runtime.status, "Wrap on");
}

#[test]
fn render_command_palette_overlay_shows_matching_commands() {
    let palette = CommandPaletteState {
        query: String::from("reader"),
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_command_palette_overlay(&mut output, &palette, 10, 0, 90, 1, EditorTheme::default(), false)
        .expect("render command palette");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("Command: reader"));
    assert!(output.contains("Reader mode"));
    assert!(output.contains("Reader slower"));
    assert!(output.contains("Reader faster"));
}

#[test]
fn render_command_palette_overlay_without_colors_skips_color_escapes() {
    let palette = CommandPaletteState {
        query: String::from("reader"),
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    write_command_palette_overlay(
        &mut output,
        &palette,
        10,
        0,
        90,
        1,
        EditorTheme::default(),
        true,
    )
    .expect("render command palette");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}

#[test]
fn render_file_sidebar_without_colors_skips_color_escapes() {
    let sidebar = FileSidebarState {
        current_dir: PathBuf::from("/tmp"),
        entries: vec![
            kfnotepad::FileSidebarEntry {
                label: String::from("../"),
                path: PathBuf::from("/"),
                kind: kfnotepad::FileSidebarEntryKind::Parent,
            },
            kfnotepad::FileSidebarEntry {
                label: String::from("notes.md"),
                path: PathBuf::from("/tmp/notes.md"),
                kind: kfnotepad::FileSidebarEntryKind::File,
            },
        ],
        selected: 1,
        scroll: 0,
    };
    let mut output = Vec::new();

    render_file_sidebar(
        &mut output,
        &sidebar,
        8,
        EditorTheme::default(),
        true,
    )
    .expect("render file sidebar");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(!output.contains("\x1b[38;"));
    assert!(!output.contains("\x1b[48;"));
}

#[test]
fn render_expands_tabs_to_terminal_columns() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("a\tb\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 2 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        20,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("a   b"));
    assert!(output.ends_with("\u{1b}[2;9H"));
}

#[test]
fn render_positions_cursor_after_wide_character() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("界x\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 1 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        20,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("界x"));
    assert!(output.ends_with("\u{1b}[2;7H"));
}

#[test]
fn render_keeps_combining_mark_at_zero_width() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("e\u{301}x\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 2 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        20,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("e\u{301}x"));
    assert!(output.ends_with("\u{1b}[2;6H"));
}

#[test]
fn render_uses_horizontal_offset_for_long_lines() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 10 },
            viewport_start: 0,
            horizontal_offset: 6,
            visible_rows: 1,
            status: "status",
            settings: EditorSettings::default(),
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        10,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("ghijk"));
    assert!(!output.contains("abcde"));
    assert!(output.ends_with("\u{1b}[2;9H"));
}

#[test]
fn render_wraps_long_lines_when_enabled() {
    let document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("abcdefghijklmnop\n"),
    };
    let mut output = Vec::new();
    let highlighter = SyntaxHighlighter::default();

    render_editor_with_width(
        &mut output,
        &document,
        EditorView {
            cursor: Cursor { row: 0, column: 8 },
            viewport_start: 0,
            horizontal_offset: 0,
            visible_rows: 3,
            status: "status",
            settings: EditorSettings {
                wrap_lines: true,
                ..EditorSettings::default()
            },
            menu: None,
            sidebar_width: 0,
            tab_strip: &[],
            search_highlight: None,
        },
        &highlighter,
        10,
    )
    .expect("render editor");

    let output = String::from_utf8(output).expect("rendered output is UTF-8");
    assert!(output.contains("abcdef"));
    assert!(output.contains("ghijkl"));
    assert!(output.contains("mnop"));
    assert!(output.ends_with("\u{1b}[3;7H"));
}

#[test]
fn wrap_prefers_word_boundaries() {
    assert_eq!(
        wrapped_chunk_texts("alpha beta gamma", 10),
        vec!["alpha beta".to_string(), "gamma".to_string()]
    );
}

#[test]
fn wrap_falls_back_to_character_chunks_for_long_words() {
    assert_eq!(
        wrapped_chunk_texts("superlongword", 5),
        vec!["super".to_string(), "longw".to_string(), "ord".to_string()]
    );
}

#[test]
fn wrap_preserves_leading_indentation_on_first_visual_row() {
    assert_eq!(
        wrapped_chunk_texts("    let value = call();", 12),
        vec![
            "    let".to_string(),
            "value =".to_string(),
            "call();".to_string()
        ]
    );
    assert_eq!(
        wrapped_line_chunks("    let value = call();", 12)[0].start_column,
        0
    );
    assert_eq!(
        wrapped_line_chunks("    let value = call();", 12)[1].start_column,
        8
    );
}

fn wrapped_chunk_texts(line: &str, width: usize) -> Vec<String> {
    wrapped_line_chunks(line, width)
        .into_iter()
        .map(|chunk| chunk.text.to_string())
        .collect()
}
