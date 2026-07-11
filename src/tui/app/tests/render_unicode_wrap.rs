use super::*;
use crate::tui::render::*;

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
