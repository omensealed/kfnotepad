#[cfg(test)]
mod tests {
    use super::*;
    use kfnotepad::TextBuffer;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn no_color_enabled_follows_non_empty_no_color_env() {
        assert!(!no_color_enabled_from(None));
        assert!(!no_color_enabled_from(Some(OsString::from(""))));
        assert!(no_color_enabled_from(Some(OsString::from("1"))));
        assert!(no_color_enabled_from(Some(OsString::from("true"))));
    }

    #[test]
    fn render_editor_without_colors_skips_color_escapes() {
        let document = TextDocument {
            path: PathBuf::from("note.txt"),
            buffer: TextBuffer::from_text("hello world\n"),
        };
        let mut output = Vec::new();
        let highlighter = SyntaxHighlighter::default();

        render_editor_with_width_and_color(
            &mut output,
            &document,
            EditorView {
                cursor: Cursor { row: 0, column: 0 },
                viewport_start: 0,
                horizontal_offset: 0,
                visible_rows: 4,
                status: "status",
                settings: EditorSettings::default(),
                menu: None,
                sidebar_width: 0,
                tab_strip: &[],
                search_highlight: None,
            },
            &highlighter,
            80,
            true,
        )
        .expect("render editor");

        let output = String::from_utf8(output).expect("render output is utf-8");
        assert!(!output.contains("\x1b[38;"));
        assert!(!output.contains("\x1b[48;"));
    }

    #[test]
    fn text_buffer_revision_tracks_content_changes_for_render_caches() {
        let mut buffer = TextBuffer::from_text("hello\n");
        let initial_revision = buffer.edit_revision();

        buffer.insert_char(0, 5, '!').expect("insert char");
        assert!(buffer.edit_revision() > initial_revision);
        let edited_revision = buffer.edit_revision();

        buffer.mark_clean();
        assert_eq!(buffer.edit_revision(), edited_revision);

        buffer.delete_char(0, 5).expect("delete char");
        assert!(buffer.edit_revision() > edited_revision);
    }

    #[test]
    fn tui_syntax_cache_reuses_viewport_and_invalidates_after_edit() {
        let mut document = TextDocument {
            path: PathBuf::from("main.rs"),
            buffer: TextBuffer::from_text(&format!("/* start\n{}*/\n", "inside\n".repeat(20))),
        };
        let highlighter = SyntaxHighlighter::default();
        let mut cache = TuiSyntaxHighlightCache::default();

        let highlighted = cache.highlight(&document, 10, 2, &highlighter);
        assert!(highlighted.iter().all(Option::is_some));

        cache.lines = vec![None];
        assert_eq!(cache.highlight(&document, 10, 2, &highlighter), vec![None]);

        document
            .buffer
            .insert_char(0, 0, ' ')
            .expect("edit document before viewport");
        let highlighted_after_edit = cache.highlight(&document, 10, 2, &highlighter);
        assert!(highlighted_after_edit.iter().all(Option::is_some));
    }

    #[test]
    fn wrapped_line_count_matches_borrowed_chunks_without_materializing_text() {
        for (line, width) in [
            ("", 8),
            ("alpha beta gamma", 10),
            ("superlongword", 5),
            ("    let value = call();", 12),
            ("wide 界界 text", 6),
        ] {
            assert_eq!(
                wrapped_line_chunk_count(line, width),
                wrapped_line_chunks(line, width).len()
            );
        }

        let source = String::from("alpha beta gamma");
        let chunks = wrapped_line_chunks(&source, 10);
        assert_eq!(chunks[0].text, "alpha beta");
        assert_eq!(chunks[1].text, "gamma");
    }

    #[test]
    fn wrapped_line_chunks_preserve_grapheme_clusters() {
        let flag = "🇺🇸";
        let chunks = wrapped_line_chunks(&format!("{flag}x"), 1)
            .into_iter()
            .map(|chunk| chunk.text.to_string())
            .collect::<Vec<_>>();
        assert_eq!(chunks, vec![flag.to_string(), "x".to_string()]);

        let family = "👨‍👩‍👧‍👦";
        let chunks = wrapped_line_chunks(&format!("{family}x"), 1)
            .into_iter()
            .map(|chunk| chunk.text.to_string())
            .collect::<Vec<_>>();
        assert_eq!(chunks, vec![family.to_string(), "x".to_string()]);

        let chunks = wrapped_line_chunks("e\u{301}x", 1)
            .into_iter()
            .map(|chunk| chunk.text.to_string())
            .collect::<Vec<_>>();
        assert_eq!(chunks, vec!["e\u{301}".to_string(), "x".to_string()]);
    }

    #[test]
    fn line_window_with_search_preserves_grapheme_clusters() {
        let mut output = Vec::new();
        let mut display_column = 0;
        let mut source_column = 0;
        let mut remaining = 8;
        let search_range = 0..2;

        print_line_window_with_search(
            &mut output,
            LineWindowSearchView {
                text: "🇺🇸x",
                start_column: 1,
                display_column: &mut display_column,
                source_column: &mut source_column,
                remaining_columns: &mut remaining,
                search_ranges: std::slice::from_ref(&search_range),
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
        .expect("print grapheme window");

        let output = String::from_utf8(output).expect("line output is UTF-8");
        assert!(output.contains("🇺🇸"));
        assert!(output.contains("x"));
        assert!(output.starts_with("\u{1b}["));
    }

    #[test]
    fn display_column_helpers_preserve_grapheme_boundaries() {
        assert_eq!(line_display_width_until("🇺🇸x", 0), 0);
        assert_eq!(line_display_width_until("🇺🇸x", 1), text_display_width("🇺🇸"));
        assert_eq!(line_display_width_until("🇺🇸x", 2), text_display_width("🇺🇸"));
        assert_eq!(char_column_for_display_column("🇺🇸x", 0), 0);
        assert_eq!(char_column_for_display_column("🇺🇸x", 1), 2);

        assert_eq!(line_display_width_until("e\u{301}x", 1), 1);
        assert_eq!(line_display_width_until("e\u{301}x", 2), 1);
        assert_eq!(char_column_for_display_column("e\u{301}x", 0), 0);
        assert_eq!(char_column_for_display_column("e\u{301}x", 1), 2);
    }

    #[test]
    fn highlighted_segments_preserve_grapheme_clusters() {
        let style_a = SyntectStyle {
            foreground: syntect::highlighting::Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            background: syntect::highlighting::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            font_style: syntect::highlighting::FontStyle::empty(),
        };
        let style_b = SyntectStyle {
            foreground: syntect::highlighting::Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
            ..style_a
        };

        let segments = grapheme_safe_highlight_segments(vec![
            (style_a, "🇺".to_string()),
            (style_b, "🇸".to_string()),
            (style_a, " e".to_string()),
            (style_b, "\u{301}".to_string()),
            (style_a, "x".to_string()),
        ]);

        assert_eq!(
            segments
                .iter()
                .map(|(_style, text)| text.as_str())
                .collect::<Vec<_>>(),
            vec!["🇺🇸 e\u{301}x"]
        );
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
                "e\u{301}x",
                "\u{301}",
                SearchMode {
                    case_sensitive: true,
                },
            ),
            vec![0..2]
        );
        assert_eq!(
            search_match_ranges(
                "aßb SS",
                "ss",
                SearchMode {
                    case_sensitive: false,
                },
            ),
            vec![1..2, 4..6]
        );
        assert_eq!(
            search_match_ranges(
                "İstanbul",
                "i",
                SearchMode {
                    case_sensitive: false,
                },
            ),
            vec![0..1]
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
        assert_eq!(
            search_match_ranges(
                "İx",
                "\u{307}",
                SearchMode {
                    case_sensitive: false,
                },
            ),
            vec![0..1]
        );
    }
}
