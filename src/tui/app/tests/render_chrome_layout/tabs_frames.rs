use super::*;

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
