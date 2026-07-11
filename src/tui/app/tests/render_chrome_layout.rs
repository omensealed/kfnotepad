use super::*;
use crate::tui::menu::*;
use crate::tui::render::*;

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
