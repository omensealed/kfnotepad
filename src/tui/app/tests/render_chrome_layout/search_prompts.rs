use super::*;

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
