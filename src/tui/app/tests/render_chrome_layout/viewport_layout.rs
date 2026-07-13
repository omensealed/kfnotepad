use super::*;

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
