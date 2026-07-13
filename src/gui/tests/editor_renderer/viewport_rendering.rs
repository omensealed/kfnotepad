use super::*;

#[test]
fn gui_editor_viewport_slice_uses_same_viewport_as_gutter() {
    let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

    adapter.apply(GuiEditorCommand::ScrollViewportLines(2));
    let render = adapter.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16);

    assert_eq!(
        render.line_numbers.gutter_start,
        render.viewport_slice.first_line
    );
    assert_eq!(render.viewport_slice.line_count, 100);
    assert_eq!(
        render.viewport_slice.lines.first(),
        Some(&GuiEditorViewportLine {
            number: 3,
            text: "3".to_string(),
            cursor_column: Some(0),
            selection: None,
            syntax_segments: None,
        })
    );
    assert_eq!(
        render.viewport_slice.lines.last(),
        Some(&GuiEditorViewportLine {
            number: 34,
            text: "34".to_string(),
            cursor_column: None,
            selection: None,
            syntax_segments: None,
        })
    );
    assert_eq!(
        render.viewport_slice.lines.len(),
        GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
    );
}

#[test]
fn gui_editor_viewport_slice_preserves_trailing_blank_line() {
    let adapter = GuiEditorAdapter::from_text("one\ntwo\n");
    let render = adapter.render_state(5, 16);

    assert_eq!(
        render.viewport_slice,
        GuiEditorViewportSlice {
            line_count: 3,
            first_line: 1,
            lines: vec![
                GuiEditorViewportLine {
                    number: 1,
                    text: "one".to_string(),
                    cursor_column: Some(0),
                    selection: None,
                    syntax_segments: None,
                },
                GuiEditorViewportLine {
                    number: 2,
                    text: "two".to_string(),
                    cursor_column: None,
                    selection: None,
                    syntax_segments: None,
                },
                GuiEditorViewportLine {
                    number: 3,
                    text: String::new(),
                    cursor_column: None,
                    selection: None,
                    syntax_segments: None,
                },
            ],
        }
    );
}

#[test]
fn gui_editor_viewport_slice_exposes_replacement_selection_spans() {
    let text = "zero\none\ntwo\nthree";
    let line_count = TextBuffer::from_text(text).line_count();
    let render = gui_editor_viewport_slice(
        text,
        line_count,
        GuiEditorViewportState {
            first_line: 2,
            visible_lines: 3,
        },
        DocumentCursor { row: 1, column: 0 },
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 3, column: 2 },
            focus: DocumentCursor { row: 1, column: 1 },
        }),
    );

    assert_eq!(render.first_line, 2);
    assert_eq!(
        render
            .lines
            .iter()
            .map(|line| (line.number, line.selection))
            .collect::<Vec<_>>(),
        vec![
            (
                2,
                Some(GuiEditorSelectionSpan {
                    start_column: 1,
                    end_column: 3,
                }),
            ),
            (
                3,
                Some(GuiEditorSelectionSpan {
                    start_column: 0,
                    end_column: 3,
                }),
            ),
            (
                4,
                Some(GuiEditorSelectionSpan {
                    start_column: 0,
                    end_column: 2,
                }),
            ),
        ]
    );
}

#[test]
fn gui_editor_viewport_selection_span_ignores_empty_and_offscreen_ranges() {
    let document = TextDocument {
        path: PathBuf::from("selection-render.txt"),
        buffer: TextBuffer::from_text("alpha\n\nomega"),
    };
    let cursor = DocumentCursor { row: 0, column: 0 };
    let viewport = GuiEditorViewportState::new(2);

    let empty = gui_editor_viewport_slice(
        &document.buffer.to_text(),
        document.buffer.line_count(),
        viewport,
        cursor,
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 0, column: 2 },
            focus: DocumentCursor { row: 0, column: 2 },
        }),
    );
    assert!(empty.lines.iter().all(|line| line.selection.is_none()));

    let offscreen = gui_editor_viewport_slice(
        &document.buffer.to_text(),
        document.buffer.line_count(),
        viewport,
        cursor,
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 2, column: 1 },
            focus: DocumentCursor { row: 2, column: 4 },
        }),
    );
    assert!(offscreen.lines.iter().all(|line| line.selection.is_none()));

    let blank_line = gui_editor_viewport_slice(
        &document.buffer.to_text(),
        document.buffer.line_count(),
        viewport,
        cursor,
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 0, column: 3 },
            focus: DocumentCursor { row: 2, column: 2 },
        }),
    );
    assert_eq!(
        blank_line.lines[1].selection,
        Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 0,
        })
    );
}

#[test]
fn gui_editor_read_only_line_segments_mark_selected_text() {
    let line = GuiEditorViewportLine {
        number: 1,
        text: "abcdef".to_string(),
        cursor_column: None,
        selection: Some(GuiEditorSelectionSpan {
            start_column: 2,
            end_column: 5,
        }),
        syntax_segments: None,
    };

    assert_eq!(
        gui_editor_read_only_line_segments(&line),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "ab".to_string(),
                selected: false,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: "cde".to_string(),
                selected: true,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: "f".to_string(),
                selected: false,
                syntax_color: None,
            },
        ]
    );
}

#[test]
fn gui_editor_read_only_line_segments_paint_blank_selection_cell() {
    let line = GuiEditorViewportLine {
        number: 2,
        text: String::new(),
        cursor_column: None,
        selection: Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 0,
        }),
        syntax_segments: None,
    };

    assert_eq!(
        gui_editor_read_only_line_segments(&line),
        vec![GuiEditorReadOnlyLineSegment {
            text: " ".to_string(),
            selected: true,
            syntax_color: None,
        }]
    );
}

#[test]
fn gui_editor_read_only_line_segments_paint_cursor_cell() {
    let line = GuiEditorViewportLine {
        number: 1,
        text: "abc".to_string(),
        cursor_column: Some(1),
        selection: None,
        syntax_segments: None,
    };

    assert_eq!(
        gui_editor_read_only_line_segments(&line),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "a".to_string(),
                selected: false,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: "b".to_string(),
                selected: true,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: "c".to_string(),
                selected: false,
                syntax_color: None,
            },
        ]
    );

    let end_cursor = GuiEditorViewportLine {
        number: 1,
        text: "abc".to_string(),
        cursor_column: Some(3),
        selection: None,
        syntax_segments: None,
    };
    assert_eq!(
        gui_editor_read_only_line_segments(&end_cursor),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "abc".to_string(),
                selected: false,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: " ".to_string(),
                selected: true,
                syntax_color: None,
            },
        ]
    );
}

#[test]
fn gui_editor_read_only_line_segments_preserve_syntax_colors_until_overlay() {
    let keyword = Color::from_rgb8(255, 0, 80);
    let plain = Color::from_rgb8(120, 200, 255);
    let line = GuiEditorViewportLine {
        number: 1,
        text: "let x".to_string(),
        cursor_column: None,
        selection: None,
        syntax_segments: Some(vec![
            GuiEditorSyntaxSegment {
                text: "let".to_string(),
                color: keyword,
            },
            GuiEditorSyntaxSegment {
                text: " x".to_string(),
                color: plain,
            },
        ]),
    };

    assert_eq!(
        gui_editor_read_only_line_segments(&line),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "let".to_string(),
                selected: false,
                syntax_color: Some(keyword),
            },
            GuiEditorReadOnlyLineSegment {
                text: " x".to_string(),
                selected: false,
                syntax_color: Some(plain),
            },
        ]
    );

    let selected = GuiEditorViewportLine {
        selection: Some(GuiEditorSelectionSpan {
            start_column: 1,
            end_column: 4,
        }),
        ..line
    };
    assert_eq!(
        gui_editor_read_only_line_segments(&selected),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "l".to_string(),
                selected: false,
                syntax_color: Some(keyword),
            },
            GuiEditorReadOnlyLineSegment {
                text: "et ".to_string(),
                selected: true,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: "x".to_string(),
                selected: false,
                syntax_color: Some(plain),
            },
        ]
    );
}

#[test]
fn gui_editor_read_only_line_segments_preserve_grapheme_clusters_across_syntax_splits() {
    let red = Color::from_rgb8(255, 0, 80);
    let blue = Color::from_rgb8(80, 120, 255);
    let line = GuiEditorViewportLine {
        number: 1,
        text: "🇺🇸 e\u{301}x".to_string(),
        cursor_column: None,
        selection: None,
        syntax_segments: Some(vec![
            GuiEditorSyntaxSegment {
                text: "🇺".to_string(),
                color: red,
            },
            GuiEditorSyntaxSegment {
                text: "🇸".to_string(),
                color: blue,
            },
            GuiEditorSyntaxSegment {
                text: " e".to_string(),
                color: red,
            },
            GuiEditorSyntaxSegment {
                text: "\u{301}".to_string(),
                color: blue,
            },
            GuiEditorSyntaxSegment {
                text: "x".to_string(),
                color: red,
            },
        ]),
    };

    assert_eq!(
        gui_editor_read_only_line_segments(&line),
        vec![GuiEditorReadOnlyLineSegment {
            text: "🇺🇸 e\u{301}x".to_string(),
            selected: false,
            syntax_color: Some(red),
        }]
    );

    let selected = GuiEditorViewportLine {
        selection: Some(GuiEditorSelectionSpan {
            start_column: 1,
            end_column: 2,
        }),
        ..line
    };
    assert_eq!(
        gui_editor_read_only_line_segments(&selected),
        vec![
            GuiEditorReadOnlyLineSegment {
                text: "🇺🇸".to_string(),
                selected: true,
                syntax_color: None,
            },
            GuiEditorReadOnlyLineSegment {
                text: " e\u{301}x".to_string(),
                selected: false,
                syntax_color: Some(red),
            },
        ]
    );
}

#[test]
fn gui_editor_read_only_line_spans_carry_color_and_overlay_highlight() {
    let palette = gui_theme_palette(EditorThemeId::Abyss);
    let keyword = Color::from_rgb8(255, 0, 80);
    let line = GuiEditorViewportLine {
        number: 1,
        text: "let".to_string(),
        cursor_column: Some(1),
        selection: None,
        syntax_segments: Some(vec![GuiEditorSyntaxSegment {
            text: "let".to_string(),
            color: keyword,
        }]),
    };

    let spans = gui_editor_read_only_line_spans(&line, palette, false);

    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].color, Some(keyword));
    assert!(spans[0].highlight.is_none());
    assert_eq!(spans[1].color, Some(palette.background));
    assert!(spans[1].highlight.is_some());
    assert_eq!(spans[2].color, Some(keyword));
    assert!(spans[2].highlight.is_none());
}

#[test]
fn gui_editor_read_only_line_spans_use_stronger_search_highlight() {
    let palette = gui_theme_palette(EditorThemeId::Nocturne);
    let line = GuiEditorViewportLine {
        number: 1,
        text: "match".to_string(),
        cursor_column: None,
        selection: Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 5,
        }),
        syntax_segments: None,
    };

    let normal = gui_editor_read_only_line_spans(&line, palette, false);
    let search = gui_editor_read_only_line_spans(&line, palette, true);
    let normal_background = match normal[0].highlight.expect("normal highlight").background {
        Background::Color(color) => color,
        _ => panic!("expected normal color highlight"),
    };
    let search_background = match search[0].highlight.expect("search highlight").background {
        Background::Color(color) => color,
        _ => panic!("expected search color highlight"),
    };

    assert!(search_background.a > normal_background.a);
    assert_eq!(search_background.r, normal_background.r);
    assert_eq!(search_background.g, normal_background.g);
    assert_eq!(search_background.b, normal_background.b);
    assert_eq!(search[0].color, Some(palette.background));
}

#[test]
fn gui_editor_word_wrap_ranges_prefer_words_then_long_word_breaks() {
    assert_eq!(
        gui_editor_word_wrap_ranges("hello world again", 8),
        vec![(0, 6), (6, 12), (12, 17)]
    );
    assert_eq!(
        gui_editor_word_wrap_ranges("abcdefgh", 3),
        vec![(0, 3), (3, 6), (6, 8)]
    );
}

#[test]
fn gui_editor_word_wrap_ranges_use_display_width_for_wide_text_and_tabs() {
    assert_eq!(
        gui_editor_word_wrap_ranges("ab界cd", 4),
        vec![(0, 3), (3, 5)]
    );
    assert_eq!(gui_editor_word_wrap_ranges("界界", 2), vec![(0, 1), (1, 2)]);
    assert_eq!(
        gui_editor_word_wrap_ranges("a\tbc", 4),
        vec![(0, 1), (1, 2), (2, 4)]
    );
}

#[test]
fn gui_editor_word_wrap_ranges_preserve_grapheme_clusters() {
    let flag = "🇺🇸";
    assert_eq!(
        gui_editor_word_wrap_ranges(&format!("{flag}x"), 1),
        vec![(0, 2), (2, 3)]
    );

    let family = "👨‍👩‍👧‍👦";
    assert_eq!(
        gui_editor_word_wrap_ranges(&format!("{family}x"), 1),
        vec![(0, family.chars().count()), (family.chars().count(), 8)]
    );

    assert_eq!(
        gui_editor_word_wrap_ranges("e\u{301}x", 1),
        vec![(0, 2), (2, 3)]
    );
}

#[test]
fn gui_editor_visual_row_mouse_mapping_uses_display_width() {
    let settings = EditorSettings::default();
    let character_width = gui_editor_replacement_character_width(settings);

    assert_eq!(
        gui_editor_replacement_mouse_point_from_visual_row_point(
            iced::Point::new(character_width * 0.2, 0.0),
            2,
            5,
            "a界b",
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 5,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_visual_row_point(
            iced::Point::new(character_width * 3.1, 0.0),
            2,
            5,
            "a界b",
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 7,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_visual_row_point(
            iced::Point::new(character_width * 3.8, 0.0),
            2,
            5,
            "a界b",
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 8,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_visual_row_point(
            iced::Point::new(character_width * 2.0, 0.0),
            2,
            5,
            "a\tb",
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 6,
        }
    );
}

#[test]
fn gui_editor_replacement_row_height_is_fixed_from_editor_font_size() {
    let mut settings = EditorSettings {
        gui_font_size: 16,
        ..EditorSettings::default()
    };

    assert_eq!(gui_editor_replacement_row_height(settings), 21.0);

    settings.gui_font_size = 17;
    assert_eq!(gui_editor_replacement_row_height(settings), 23.0);
}

#[test]
fn gui_editor_visible_row_budget_avoids_partial_row_overflow() {
    assert_eq!(gui_editor_visible_row_budget(8.0, 20.0), 1);
    assert_eq!(gui_editor_visible_row_budget(40.0, 20.0), 2);
    assert_eq!(gui_editor_visible_row_budget(47.9, 20.0), 2);
    assert_eq!(gui_editor_visible_row_budget(60.0, 20.0), 3);
}

#[test]
fn gui_editor_read_only_visual_rows_keep_gutter_and_cursor_snug() {
    let line = GuiEditorViewportLine {
        number: 7,
        text: "alpha beta gamma".to_string(),
        cursor_column: Some(8),
        selection: Some(GuiEditorSelectionSpan {
            start_column: 6,
            end_column: 11,
        }),
        syntax_segments: None,
    };

    let rows = gui_editor_read_only_visual_rows(&[line], 7, Wrapping::Word, 6);

    assert_eq!(
        rows.iter()
            .map(|row| (row.line.text.as_str(), row.show_line_number))
            .collect::<Vec<_>>(),
        vec![("alpha ", true), ("beta ", false), ("gamma", false)]
    );
    assert_eq!(rows[1].viewport_row, 0);
    assert_eq!(rows[1].source_column_start, 6);
    assert_eq!(rows[1].line.cursor_column, Some(2));
    assert_eq!(
        rows[1].line.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 5,
        })
    );
}

#[test]
fn gui_editor_viewport_line_slice_preserves_grapheme_clusters() {
    let line = GuiEditorViewportLine {
        number: 1,
        text: "a🇺🇸b e\u{301}x".to_string(),
        cursor_column: Some(2),
        selection: Some(GuiEditorSelectionSpan {
            start_column: 2,
            end_column: 6,
        }),
        syntax_segments: None,
    };

    let flag_slice = gui_editor_viewport_line_slice(&line, 2, 3);
    assert_eq!(flag_slice.text, "🇺🇸");
    assert_eq!(flag_slice.cursor_column, Some(2));
    assert_eq!(
        flag_slice.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 2,
        })
    );

    let combining_slice = gui_editor_viewport_line_slice(&line, 6, 7);
    assert_eq!(combining_slice.text, "e\u{301}");
    assert_eq!(
        combining_slice.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 0,
            end_column: 2,
        })
    );
}

#[test]
fn gui_editor_read_only_render_model_scrolls_text_and_gutter_together() {
    let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

    adapter.apply(GuiEditorCommand::ScrollViewportLines(2));
    let render = adapter.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16);
    let model = gui_editor_read_only_render_model(&render.viewport_slice);

    assert_eq!(model.first_line, 3);
    assert_eq!(model.line_count, 100);
    assert_eq!(
        model.gutter_text,
        gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES)
    );
    assert!(model.body_text.starts_with("3\n4\n5\n"));
    assert!(model.body_text.ends_with("\n34"));
    assert_eq!(model.cursor_row_in_view, Some(0));
    assert_eq!(model.cursor_column, Some(0));
}
