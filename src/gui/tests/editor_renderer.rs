#[test]
fn gui_editor_rendering_helpers_reflect_line_number_and_wrap_preferences() {
    assert_eq!(gui_editor_wrapping(false), Wrapping::None);
    assert_eq!(gui_editor_wrapping(true), Wrapping::WordOrGlyph);
    assert_eq!(
        gui_editor_effective_wrapping(true, false),
        Wrapping::WordOrGlyph
    );
    assert_eq!(
        gui_editor_effective_wrapping(true, true),
        Wrapping::WordOrGlyph
    );
    assert_eq!(gui_editor_effective_wrapping(false, true), Wrapping::None);
    assert_eq!(gui_line_number_gutter_text(1, 0, 3), "1");
    assert_eq!(gui_line_number_gutter_text(3, 10, 4), "3\n4\n5\n6");
    assert_eq!(gui_line_number_gutter_text(99, 10, 4), "10");
    assert!(gui_line_number_gutter_width(100, 16) > gui_line_number_gutter_width(9, 16));
    assert!(gui_line_number_gutter_width(236, 16) < 40.0);
    assert!(gui_line_number_gutter_width(236, 20) > gui_line_number_gutter_width(236, 16));
    assert_eq!(gui_left_panel_width(false, 260.0), 0.0);
    assert_eq!(gui_left_panel_width(true, 260.0), 260.0);
    assert_eq!(gui_left_panel_width(true, 999.0), GUI_BROWSER_WIDTH_MAX);
}

#[test]
fn gui_editor_surface_model_captures_backend_replacement_inputs() {
    let document = TextDocument {
        path: PathBuf::from("surface.rs"),
        buffer: TextBuffer::from_text("fn main() {}\nsecond\n"),
    };
    let mut adapter = GuiEditorAdapter::from_text("fn main() {}\nsecond\n");
    adapter.move_to(DocumentCursor { row: 1, column: 0 });
    let settings = EditorSettings {
        show_line_numbers: true,
        wrap_lines: true,
        gui_font_family: GuiFontFamily::FiraCode,
        gui_font_size: 18,
        theme_id: EditorThemeId::Terror,
        ..EditorSettings::default()
    };
    let highlighter = SyntaxHighlighter::default();

    let cache = gui_test_syntax_cache_for_document(&highlighter, &document, 8);
    let surface =
        gui_editor_surface_model(settings, &document, &adapter, &highlighter, Some(&cache));

    assert_eq!(surface.content.text(), "fn main() {}\nsecond\n");
    assert_eq!(surface.editor_size, 18);
    assert_eq!(surface.wrapping, Wrapping::WordOrGlyph);
    assert_eq!(surface.syntax_token, "rs");
    assert_eq!(
        surface.line_numbers,
        Some(GuiEditorLineNumberSnapshot {
            line_count: 3,
            gutter_start: 1,
            text: "1\n2\n3".to_string(),
            width: gui_line_number_gutter_width(3, 18),
        })
    );
    let mut viewport_without_syntax = surface.viewport_slice.clone();
    assert!(viewport_without_syntax.lines[0].syntax_segments.is_some());
    for line in &mut viewport_without_syntax.lines {
        line.syntax_segments = None;
    }
    assert_eq!(
        viewport_without_syntax,
        GuiEditorViewportSlice {
            line_count: 3,
            first_line: 1,
            lines: vec![
                GuiEditorViewportLine {
                    number: 1,
                    text: "fn main() {}".to_string(),
                    cursor_column: None,
                    selection: None,
                    syntax_segments: None,
                },
                GuiEditorViewportLine {
                    number: 2,
                    text: "second".to_string(),
                    cursor_column: Some(0),
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

    let hidden_numbers = gui_editor_surface_model(
        EditorSettings {
            show_line_numbers: false,
            ..settings
        },
        &document,
        &adapter,
        &highlighter,
        Some(&cache),
    );
    assert_eq!(hidden_numbers.line_numbers, None);
    assert_eq!(hidden_numbers.wrapping, Wrapping::WordOrGlyph);
}

#[test]
fn gui_editor_surface_model_renders_beyond_stale_logical_viewport_height() {
    let text = numbered_lines(100);
    let document = TextDocument {
        path: PathBuf::from("surface-long.txt"),
        buffer: TextBuffer::from_text(&text),
    };
    let mut adapter = GuiEditorAdapter::from_text(&text);
    adapter.apply(GuiEditorCommand::ScrollViewportLines(30));
    let highlighter = SyntaxHighlighter::default();

    let surface = gui_editor_surface_model(
        EditorSettings::default(),
        &document,
        &adapter,
        &highlighter,
        None,
    );

    assert_eq!(
        adapter.viewport.visible_lines,
        GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
    );
    assert_eq!(surface.viewport_slice.first_line, 31);
    assert_eq!(
        surface.viewport_slice.lines.first().map(|line| line.number),
        Some(31)
    );
    assert_eq!(
        surface.viewport_slice.lines.last().map(|line| line.number),
        Some(100)
    );
    assert_eq!(surface.viewport_slice.lines.len(), 70);
    assert_eq!(
        surface
            .line_numbers
            .as_ref()
            .map(|numbers| numbers.text.clone()),
        Some(gui_line_number_gutter_text(
            31,
            100,
            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES
        ))
    );
}

#[test]
fn gui_editor_surface_model_bounds_large_document_source_slice() {
    let text = numbered_lines(2_000);
    let document = TextDocument {
        path: PathBuf::from("surface-large.txt"),
        buffer: TextBuffer::from_text(&text),
    };
    let mut adapter = GuiEditorAdapter::from_text(&text);
    adapter.apply(GuiEditorCommand::ScrollViewportLines(1_200));
    let highlighter = SyntaxHighlighter::default();

    let surface = gui_editor_surface_model(
        EditorSettings::default(),
        &document,
        &adapter,
        &highlighter,
        None,
    );

    assert_eq!(surface.viewport_slice.first_line, 1_201);
    assert_eq!(
        surface.viewport_slice.lines.len(),
        GUI_EDITOR_RENDER_LINE_BUDGET
    );
    assert_eq!(
        surface.viewport_slice.lines.last().map(|line| line.number),
        Some(1_200 + GUI_EDITOR_RENDER_LINE_BUDGET)
    );
}

#[test]
fn gui_syntax_cache_extends_to_visible_large_document_scroll() {
    let temp = TempArea::new("gui-syntax-cache-scroll");
    let path = temp.path("large.rs");
    let text = (0..2_000)
        .map(|index| format!("fn function_{index}() -> usize {{ {index} }}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, text).expect("write large rust file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![path],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    let initial_until = state
        .syntax_caches
        .get(&tile_id)
        .map(|cache| cache.highlighted_until)
        .expect("initial syntax cache");

    assert_eq!(initial_until, GUI_EDITOR_RENDER_LINE_BUDGET);
    assert!(
        state
            .syntax_caches
            .get(&tile_id)
            .and_then(|cache| cache.lines.first())
            .and_then(|line| line.as_ref())
            .is_some(),
        "Rust file should keep cached syntax segments"
    );

    let _ = update(&mut state, Message::ScrollActiveEditorViewport(80));

    let cache = state
        .syntax_caches
        .get(&tile_id)
        .expect("extended syntax cache");
    assert_eq!(cache.highlighted_until, initial_until + 80);
    assert_eq!(cache.lines.len(), initial_until + 80);
}

#[test]
fn gui_syntax_cache_scrolls_real_large_source_incrementally() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/gui/app.rs");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![source_path],
        },
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    let tile_id = state.workspace.active_tile().id;
    let line_count = state.workspace.active_tile().document.buffer.line_count();
    let initial_until = state
        .syntax_caches
        .get(&tile_id)
        .map(|cache| cache.highlighted_until)
        .expect("initial syntax cache");
    let started = Instant::now();

    for _ in 0..50 {
        let _ = update(&mut state, Message::ScrollActiveEditorViewport(20));
    }

    let elapsed = started.elapsed();
    let cache = state
        .syntax_caches
        .get(&tile_id)
        .expect("extended syntax cache");
    let expected_until = (initial_until + 1_000).min(line_count);
    assert_eq!(cache.highlighted_until, expected_until);
    assert!(cache.highlighted_until <= line_count);
    eprintln!(
        "large source incremental scroll: {} cached lines of {line_count} in {:?}",
        cache.highlighted_until, elapsed
    );
}

#[test]
fn gui_syntax_cache_rebuilds_after_replacement_edit() {
    let temp = TempArea::new("gui-syntax-cache-edit");
    let path = temp.path("large.rs");
    let text = (0..700)
        .map(|index| format!("fn function_{index}() -> usize {{ {index} }}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, text).expect("write rust file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![path],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    let original_line_count = state.workspace.active_tile().document.buffer.line_count();

    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertNewline]),
    );

    let cache = state
        .syntax_caches
        .get(&tile_id)
        .expect("rebuilt syntax cache");
    assert_eq!(cache.line_count, original_line_count + 1);
    assert_eq!(cache.highlighted_until, GUI_EDITOR_RENDER_LINE_BUDGET);
}

#[test]
fn gui_editor_adapter_exposes_parity_boundary_without_changing_backend() {
    let mut adapter = GuiEditorAdapter::from_text("one\ntwo\nthree\n");

    assert_eq!(adapter.text(), "one\ntwo\nthree\n");
    assert_eq!(adapter.line_count(), 4);
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );

    adapter.apply(GuiEditorCommand::MoveTo(DocumentCursor {
        row: 1,
        column: 2,
    }));
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 1, column: 2 }
    );

    let render = adapter.render_state(3, 16);
    assert_eq!(render.content.text(), "one\ntwo\nthree\n");
    assert_eq!(
        render.line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 4,
            gutter_start: 1,
            text: "1\n2\n3".to_string(),
            width: gui_line_number_gutter_width(4, 16),
        }
    );

    adapter.move_to(DocumentCursor { row: 1, column: 0 });
    adapter.select_right_chars(3);
    assert_eq!(adapter.selection().as_deref(), Some("two"));

    adapter.apply(GuiEditorCommand::SelectAll);
    assert_eq!(adapter.selection().as_deref(), Some("one\ntwo\nthree\n"));

    adapter.apply(GuiEditorCommand::Paste("alpha".to_string()));
    assert_eq!(adapter.text(), "alpha");
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 0, column: 5 }
    );

    adapter.apply(GuiEditorCommand::MoveTo(DocumentCursor {
        row: 0,
        column: 0,
    }));
    adapter.apply(GuiEditorCommand::SelectRightChars(1));
    adapter.apply(GuiEditorCommand::Delete);
    assert_eq!(adapter.text(), "lpha");
}

#[test]
fn gui_editor_adapter_select_right_chars_snaps_to_grapheme_boundaries() {
    let mut adapter = GuiEditorAdapter::from_text("🇺🇸e\u{301}x");

    adapter.move_to(DocumentCursor { row: 0, column: 1 });
    adapter.select_right_chars(1);

    assert_eq!(
        adapter
            .replacement_selection
            .expect("replacement selection")
            .normalized(),
        (
            DocumentCursor { row: 0, column: 0 },
            DocumentCursor { row: 0, column: 2 },
        )
    );
    assert_eq!(adapter.selection().as_deref(), Some("🇺🇸"));

    adapter.apply(GuiEditorCommand::MoveTo(DocumentCursor {
        row: 0,
        column: 3,
    }));
    adapter.select_right_chars(1);
    assert_eq!(
        adapter
            .replacement_selection
            .expect("replacement selection")
            .normalized(),
        (
            DocumentCursor { row: 0, column: 2 },
            DocumentCursor { row: 0, column: 4 },
        )
    );
    assert_eq!(adapter.selection().as_deref(), Some("e\u{301}"));
}

#[test]
fn gui_editor_adapter_viewport_keeps_cursor_visible_for_gutter() {
    let mut adapter = GuiEditorAdapter::from_text("one\ntwo\nthree\nfour\nfive");

    assert_eq!(
        adapter.render_state(3, 16).line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 5,
            gutter_start: 1,
            text: "1\n2\n3".to_string(),
            width: gui_line_number_gutter_width(5, 16),
        }
    );

    for _ in 0..4 {
        adapter.apply(GuiEditorCommand::IcedAction(text_editor::Action::Move(
            text_editor::Motion::Down,
        )));
    }
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 4, column: 0 }
    );
    assert_eq!(
        adapter.render_state(3, 16).line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 5,
            gutter_start: 3,
            text: "3\n4\n5".to_string(),
            width: gui_line_number_gutter_width(5, 16),
        }
    );

    for _ in 0..3 {
        adapter.apply(GuiEditorCommand::IcedAction(text_editor::Action::Move(
            text_editor::Motion::Up,
        )));
    }
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(
        adapter.render_state(3, 16).line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 5,
            gutter_start: 1,
            text: "1\n2\n3".to_string(),
            width: gui_line_number_gutter_width(5, 16),
        }
    );
}

#[test]
fn gui_editor_adapter_scrolls_viewport_and_clamps_cursor_to_visible_lines() {
    let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

    adapter.apply(GuiEditorCommand::ScrollViewportLines(2));

    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 2, column: 0 }
    );
    assert_eq!(
        adapter
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 3,
            text: gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );

    adapter.apply(GuiEditorCommand::ScrollViewportLines(99));

    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 68, column: 0 }
    );
    assert_eq!(
        adapter
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 69,
            text: gui_line_number_gutter_text(69, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );

    adapter.apply(GuiEditorCommand::ScrollViewportLines(-99));

    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor { row: 31, column: 0 }
    );
    assert_eq!(
        adapter
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 1,
            text: gui_line_number_gutter_text(1, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );
}

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

#[test]
fn gui_editor_replacement_input_model_edits_and_moves_shared_document() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement.txt"),
        buffer: TextBuffer::from_text("ab\ncd"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertChar('X'),
    );
    assert_eq!(document.buffer.to_text(), "aXb\ncd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
    );
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertNewline,
    );
    assert_eq!(document.buffer.to_text(), "aXb\n\ncd");
    assert_eq!(cursor, DocumentCursor { row: 1, column: 0 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );
    assert_eq!(document.buffer.to_text(), "aXb\ncd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteForward,
    );
    assert_eq!(document.buffer.to_text(), "aXbcd");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });
    assert_eq!(viewport.first_line, 1);
}

#[test]
fn gui_editor_replacement_overwrite_replaces_whole_grapheme_cluster() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement-overwrite-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸e\u{301}!"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input_with_mode(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        true,
        GuiEditorReplacementInput::InsertChar('x'),
    );
    assert_eq!(document.buffer.to_text(), "axe\u{301}!");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });

    apply_gui_editor_replacement_input_with_mode(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        true,
        GuiEditorReplacementInput::InsertChar('y'),
    );
    assert_eq!(document.buffer.to_text(), "axy!");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 3 });
}

#[test]
fn gui_editor_replacement_input_model_keeps_viewport_and_gutter_synced() {
    let mut document = TextDocument {
        path: PathBuf::from("replacement-long.txt"),
        buffer: TextBuffer::from_text(&numbered_lines(100)),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::ScrollViewportLines(2),
    );

    let slice = gui_editor_viewport_slice(
        &document.buffer.to_text(),
        document.buffer.line_count(),
        viewport,
        cursor,
        None,
    );
    let model = gui_editor_read_only_render_model(&slice);

    assert_eq!(cursor, DocumentCursor { row: 2, column: 0 });
    assert_eq!(slice.first_line, 3);
    assert_eq!(
        model.gutter_text,
        gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES)
    );
    assert!(model.body_text.starts_with("3\n4\n5\n"));
}

#[test]
fn gui_editor_replacement_selection_extracts_same_line_and_multiline_text() {
    let document = TextDocument {
        path: PathBuf::from("selection.txt"),
        buffer: TextBuffer::from_text("abécd\nsecond\nthird"),
    };

    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 1 },
                focus: DocumentCursor { row: 0, column: 4 },
            },
        )
        .as_deref(),
        Some("béc")
    );
    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 1, column: 3 },
                focus: DocumentCursor { row: 0, column: 2 },
            },
        )
        .as_deref(),
        Some("écd\nsec")
    );
}

#[test]
fn gui_editor_replacement_selection_expands_to_grapheme_boundaries() {
    let flag = "🇺🇸";
    let mut document = TextDocument {
        path: PathBuf::from("selection-grapheme.txt"),
        buffer: TextBuffer::from_text(&format!("{flag}x\n")),
    };

    assert_eq!(
        gui_editor_replacement_selected_text(
            &document,
            GuiEditorReplacementSelection {
                anchor: DocumentCursor { row: 0, column: 1 },
                focus: DocumentCursor { row: 0, column: 2 },
            },
        )
        .as_deref(),
        Some(flag)
    );

    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 1 },
        focus: DocumentCursor { row: 0, column: 2 },
    });
    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );

    assert_eq!(document.buffer.to_text(), "x");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 0 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_multiline_selection_expands_mixed_grapheme_boundaries() {
    let mut document = TextDocument {
        path: PathBuf::from("selection-mixed-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸b\nc👩‍💻d e\u{301}f"),
    };
    let reversed_selection = GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 1, column: 7 },
        focus: DocumentCursor { row: 0, column: 2 },
    };

    assert_eq!(
        gui_editor_replacement_selected_text(&document, reversed_selection).as_deref(),
        Some("🇺🇸b\nc👩‍💻d e\u{301}")
    );

    let mut cursor = DocumentCursor { row: 1, column: 7 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(reversed_selection);
    assert_eq!(
        gui_editor_replacement_cut_selection(
            &mut document,
            &mut cursor,
            &mut viewport,
            &mut selection,
        )
        .as_deref(),
        Some("🇺🇸b\nc👩‍💻d e\u{301}")
    );

    assert_eq!(document.buffer.to_text(), "af");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_selection_replaces_and_deletes_selected_ranges() {
    let mut document = TextDocument {
        path: PathBuf::from("selection-edit.txt"),
        buffer: TextBuffer::from_text("abc\ndef"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 1 },
            focus: DocumentCursor { row: 1, column: 2 },
        },
    );
    assert_eq!(cursor, DocumentCursor { row: 1, column: 2 });
    assert!(selection.is_some());

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::InsertChar('X'),
    );
    assert_eq!(document.buffer.to_text(), "aXf");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 1 },
            focus: DocumentCursor { row: 0, column: 2 },
        },
    );
    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteBackward,
    );
    assert_eq!(document.buffer.to_text(), "af");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 1 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_select_all_deletes_entire_document() {
    let mut document = TextDocument {
        path: PathBuf::from("select-all.txt"),
        buffer: TextBuffer::from_text("one\ntwo"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::SelectAll,
    );
    assert_eq!(cursor, DocumentCursor { row: 1, column: 3 });
    assert_eq!(
        gui_editor_replacement_selected_text(&document, selection.expect("selection")).as_deref(),
        Some("one\ntwo")
    );

    apply_gui_editor_replacement_input(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementInput::DeleteForward,
    );
    assert_eq!(document.buffer.to_text(), "");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 0 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_clipboard_copy_reads_selection_without_mutation() {
    let document = TextDocument {
        path: PathBuf::from("copy.txt"),
        buffer: TextBuffer::from_text("one\ntwo\nthree"),
    };
    let selection = GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 2, column: 2 },
        focus: DocumentCursor { row: 0, column: 1 },
    };

    assert_eq!(
        gui_editor_replacement_copy_selection(&document, Some(selection)).as_deref(),
        Some("ne\ntwo\nth")
    );
    assert_eq!(document.buffer.to_text(), "one\ntwo\nthree");
    assert_eq!(gui_editor_replacement_copy_selection(&document, None), None);
}

#[test]
fn gui_editor_replacement_clipboard_cut_returns_text_and_deletes_selection() {
    let mut document = TextDocument {
        path: PathBuf::from("cut.txt"),
        buffer: TextBuffer::from_text("alpha\nbeta\ngamma"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 2 },
        focus: DocumentCursor { row: 1, column: 2 },
    });

    assert_eq!(
        gui_editor_replacement_cut_selection(
            &mut document,
            &mut cursor,
            &mut viewport,
            &mut selection,
        )
        .as_deref(),
        Some("pha\nbe")
    );
    assert_eq!(document.buffer.to_text(), "alta\ngamma");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_clipboard_paste_replaces_selection_and_handles_newlines() {
    let mut document = TextDocument {
        path: PathBuf::from("paste.txt"),
        buffer: TextBuffer::from_text("hello world"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 11 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 6 },
        focus: DocumentCursor { row: 0, column: 11 },
    });

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "there\nfriend",
    );

    assert_eq!(document.buffer.to_text(), "hello there\nfriend");
    assert_eq!(cursor, DocumentCursor { row: 1, column: 6 });
    assert_eq!(selection, None);
    assert!(document.buffer.undo_last_edit());
    assert_eq!(document.buffer.to_text(), "hello world");
    assert!(!document.buffer.undo_last_edit());
}

#[test]
fn gui_editor_replacement_paste_expands_partial_grapheme_selection() {
    let mut document = TextDocument {
        path: PathBuf::from("paste-grapheme.txt"),
        buffer: TextBuffer::from_text("a🇺🇸b e\u{301}x"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 2 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 2 },
        focus: DocumentCursor { row: 0, column: 3 },
    });

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "Z",
    );

    assert_eq!(document.buffer.to_text(), "aZb e\u{301}x");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);

    selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 5 },
        focus: DocumentCursor { row: 0, column: 6 },
    });
    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "Y",
    );

    assert_eq!(document.buffer.to_text(), "aZb Yx");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 5 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_paste_advances_cursor_to_combining_grapheme_end() {
    let mut document = TextDocument {
        path: PathBuf::from("paste-combining-cursor.txt"),
        buffer: TextBuffer::from_text("e"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 1 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = None;

    gui_editor_replacement_paste_text(
        &mut document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        "\u{301}",
    );

    assert_eq!(document.buffer.to_text(), "e\u{301}");
    assert_eq!(cursor, DocumentCursor { row: 0, column: 2 });
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_mouse_point_maps_viewport_to_clamped_cursor() {
    let document = TextDocument {
        path: PathBuf::from("mouse.txt"),
        buffer: TextBuffer::from_text("one\ntwø\nthree"),
    };
    let mut viewport = GuiEditorViewportState::new(2);
    viewport.scroll_by(1, document.buffer.line_count());

    assert_eq!(
        gui_editor_replacement_cursor_from_mouse_point(
            &document.buffer,
            viewport,
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 99,
            },
        ),
        DocumentCursor { row: 1, column: 3 }
    );
    assert_eq!(
        gui_editor_replacement_cursor_from_mouse_point(
            &document.buffer,
            viewport,
            GuiEditorReplacementMousePoint {
                viewport_row: 99,
                column: 2,
            },
        ),
        DocumentCursor { row: 2, column: 2 }
    );
}

#[test]
fn gui_editor_replacement_mouse_point_snaps_to_grapheme_boundary() {
    let flag = "🇺🇸";
    let document = TextDocument {
        path: PathBuf::from("mouse.txt"),
        buffer: TextBuffer::from_text(&format!("{flag}x\n")),
    };
    let viewport = GuiEditorViewportState::new(2);

    assert_eq!(
        gui_editor_replacement_cursor_from_mouse_point(
            &document.buffer,
            viewport,
            GuiEditorReplacementMousePoint {
                viewport_row: 0,
                column: 1,
            },
        ),
        DocumentCursor { row: 0, column: 2 }
    );
}

#[test]
fn gui_editor_replacement_line_point_accounts_for_wrapped_visual_row() {
    let settings = EditorSettings::default();
    let character_width = f32::from(settings.gui_font_size) * 0.62;
    let line_height = f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT;
    let body_width = character_width * 8.0;

    assert_eq!(
        gui_editor_replacement_mouse_point_from_line_point(
            iced::Point::new(character_width * 2.2, line_height * 1.2),
            3,
            settings,
            body_width,
            Wrapping::None,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 3,
            column: 2,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_line_point(
            iced::Point::new(character_width * 2.2, line_height * 1.2),
            3,
            settings,
            body_width,
            Wrapping::WordOrGlyph,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 3,
            column: 10,
        }
    );
}

#[test]
fn gui_editor_replacement_body_point_subtracts_gutter_before_text_column_mapping() {
    let settings = EditorSettings::default();
    let character_width = gui_editor_replacement_character_width(settings);
    let row_height = gui_editor_replacement_row_height(settings);
    let snug_line =
        "See `docs/06-SECURITY.md` for the project's working threat model and release gate.";
    let lines = vec![
        "alpha".to_string(),
        "beta".to_string(),
        snug_line.to_string(),
    ];
    let viewport = GuiEditorViewportState {
        first_line: 1,
        visible_lines: 3,
    };
    let slice = gui_editor_viewport_slice_from_lines(
        &lines,
        lines.len(),
        viewport,
        DocumentCursor { row: 0, column: 0 },
        None,
    );
    let gutter_width = 42.0;
    let hit_test = GuiEditorBodyHitTest {
        columns: 120,
        visible_rows: 3,
        text_origin_x: gutter_width,
    };

    assert_eq!(
        gui_editor_replacement_mouse_point_from_body_point(
            iced::Point::new(gutter_width + 1.0, row_height * 0.25),
            &slice.lines,
            slice.first_line,
            Wrapping::WordOrGlyph,
            hit_test,
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 0,
            column: 0,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_body_point(
            iced::Point::new(gutter_width + character_width * 2.2, row_height * 1.25),
            &slice.lines,
            slice.first_line,
            Wrapping::WordOrGlyph,
            hit_test,
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 1,
            column: 2,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_body_point(
            iced::Point::new(gutter_width - 8.0, row_height * 0.25),
            &slice.lines,
            slice.first_line,
            Wrapping::WordOrGlyph,
            hit_test,
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 0,
            column: 0,
        }
    );
    assert_eq!(
        gui_editor_replacement_mouse_point_from_body_point(
            iced::Point::new(
                gutter_width + character_width * (snug_line.chars().count() as f32 - 0.1),
                row_height * 2.25,
            ),
            &slice.lines,
            slice.first_line,
            Wrapping::WordOrGlyph,
            hit_test,
            settings,
        ),
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: snug_line.chars().count(),
        }
    );
}

#[test]
fn gui_editor_replacement_mouse_click_moves_cursor_and_clears_selection() {
    let document = TextDocument {
        path: PathBuf::from("mouse-click.txt"),
        buffer: TextBuffer::from_text("alpha\nbeta"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    let mut selection = Some(GuiEditorReplacementSelection {
        anchor: DocumentCursor { row: 0, column: 1 },
        focus: DocumentCursor { row: 1, column: 2 },
    });

    gui_editor_replacement_mouse_click(
        &document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementMousePoint {
            viewport_row: 1,
            column: 99,
        },
    );

    assert_eq!(cursor, DocumentCursor { row: 1, column: 4 });
    assert_eq!(selection, None);
    assert_eq!(viewport.first_line, 1);
}

#[test]
fn gui_editor_replacement_mouse_click_preserves_visible_viewport() {
    let document = TextDocument {
        path: PathBuf::from("mouse-click-scroll.txt"),
        buffer: TextBuffer::from_text("one\ntwo\nthree\nfour\nfive"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    viewport.scroll_by(2, document.buffer.line_count());
    let mut selection = None;

    gui_editor_replacement_mouse_click(
        &document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 1,
        },
    );

    assert_eq!(cursor, DocumentCursor { row: 4, column: 1 });
    assert_eq!(viewport.first_line, 3);
    assert_eq!(selection, None);
}

#[test]
fn gui_editor_replacement_mouse_drag_sets_selection_and_cursor() {
    let document = TextDocument {
        path: PathBuf::from("mouse-drag.txt"),
        buffer: TextBuffer::from_text("zero\none\ntwo\nthree"),
    };
    let mut cursor = DocumentCursor { row: 0, column: 0 };
    let mut viewport = GuiEditorViewportState::new(3);
    viewport.scroll_by(1, document.buffer.line_count());
    let mut selection = None;

    gui_editor_replacement_mouse_drag(
        &document,
        &mut cursor,
        &mut viewport,
        &mut selection,
        DocumentCursor { row: 3, column: 5 },
        GuiEditorReplacementMousePoint {
            viewport_row: 0,
            column: 1,
        },
    );

    assert_eq!(cursor, DocumentCursor { row: 1, column: 1 });
    assert_eq!(
        selection.map(GuiEditorReplacementSelection::normalized),
        Some((
            DocumentCursor { row: 1, column: 1 },
            DocumentCursor { row: 3, column: 5 },
        ))
    );
    assert_eq!(
        gui_editor_replacement_selected_text(&document, selection.expect("selection")).as_deref(),
        Some("ne\ntwo\nthree")
    );
    assert_eq!(viewport.first_line, 2);
}

#[test]
fn gui_editor_replacement_pointer_click_updates_live_tile_cursor_without_dirtying() {
    let temp = TempArea::new("gui-replacement-pointer-click");
    let file = temp.path("pointer-click.txt");
    fs::write(&file, "alpha\nbeta").expect("write pointer click");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;

    state.replacement_editor_pointer_moved(
        pane,
        GuiEditorReplacementMousePoint {
            viewport_row: 1,
            column: 2,
        },
    );
    state.replacement_editor_pointer_pressed(pane);
    state.replacement_editor_pointer_released(pane);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 2 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 1, column: 2 }
    );
    assert_eq!(state.active_editor().replacement_selection, None);
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(state.status_message, "cursor moved");
}

#[test]
fn gui_editor_replacement_pointer_drag_updates_live_selection() {
    let temp = TempArea::new("gui-replacement-pointer-drag");
    let file = temp.path("pointer-drag.txt");
    fs::write(&file, "zero\none\ntwo\nthree").expect("write pointer drag");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;
    let line_count = state.workspace.active_tile().document.buffer.line_count();
    state
        .panes
        .get_mut(pane)
        .expect("pane")
        .editor
        .viewport
        .visible_lines = 3;
    state
        .panes
        .get_mut(pane)
        .expect("pane")
        .editor
        .viewport
        .scroll_by(1, line_count);
    assert_eq!(
        state
            .panes
            .get(pane)
            .expect("pane")
            .editor
            .viewport
            .first_line,
        2
    );

    state.replacement_editor_pointer_moved(
        pane,
        GuiEditorReplacementMousePoint {
            viewport_row: 0,
            column: 1,
        },
    );
    state.replacement_editor_pointer_pressed(pane);
    state.replacement_editor_pointer_moved(
        pane,
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 2,
        },
    );
    state.replacement_editor_pointer_released(pane);

    let selection = state
        .active_editor()
        .replacement_selection
        .expect("live selection");
    assert_eq!(
        selection.normalized(),
        (
            DocumentCursor { row: 1, column: 1 },
            DocumentCursor { row: 3, column: 2 },
        )
    );
    assert_eq!(
        gui_editor_replacement_selected_text(&state.workspace.active_tile().document, selection)
            .as_deref(),
        Some("ne\ntwo\nth")
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 3, column: 2 }
    );
    assert_eq!(
        state
            .panes
            .get(pane)
            .expect("pane")
            .editor
            .viewport
            .first_line,
        2
    );
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(state.status_message, "selected text");
}

#[test]
fn gui_editor_replacement_edge_drag_scrolls_and_extends_selection() {
    let temp = TempArea::new("gui-replacement-edge-drag");
    let file = temp.path("edge-drag.txt");
    let text = (1..=12)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file, text).expect("write edge drag");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;
    if let Some(pane_state) = state.panes.get_mut(pane) {
        pane_state.editor.viewport.visible_lines = 3;
        pane_state.editor.viewport.first_line = 1;
    }

    state.replacement_editor_pointer_moved(
        pane,
        GuiEditorReplacementMousePoint {
            viewport_row: 0,
            column: 0,
        },
    );
    state.replacement_editor_pointer_pressed(pane);
    state.replacement_editor_body_pointer_moved(
        pane,
        GuiEditorReplacementMousePoint {
            viewport_row: 2,
            column: 4,
        },
        GuiEditorDragEdge {
            pane,
            direction: 1,
            column: 4,
        },
    );
    state.replacement_editor_drag_tick();

    let pane_state = state.panes.get(pane).expect("pane");
    assert_eq!(pane_state.editor.viewport.first_line, 2);
    assert_eq!(
        pane_state.editor.document_cursor(),
        DocumentCursor { row: 3, column: 4 }
    );
    assert_eq!(
        pane_state
            .editor
            .replacement_selection
            .map(GuiEditorReplacementSelection::normalized),
        Some((
            DocumentCursor { row: 0, column: 0 },
            DocumentCursor { row: 3, column: 4 },
        ))
    );
    assert_eq!(state.status_message, "selected text");
}

#[test]
fn gui_editor_scrollbar_thumb_drag_updates_viewport_without_cursor_move() {
    let temp = TempArea::new("gui-scrollbar-thumb-drag");
    let file = temp.path("scrollbar.txt");
    let text = (1..=100)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file, text).expect("write scrollbar file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;
    if let Some(pane_state) = state.panes.get_mut(pane) {
        pane_state.editor.viewport.visible_lines = 10;
        pane_state.editor.viewport.first_line = 1;
    }

    let model = gui_editor_scrollbar_model(100, 1, 10, 200.0);
    state.replacement_editor_scrollbar_moved(pane, model.thumb_top + 2.0, model);
    state.replacement_editor_scrollbar_pressed(pane);
    state.replacement_editor_scrollbar_moved(pane, 170.0, model);

    let expected = gui_editor_scrollbar_first_line_from_thumb_y(model, 170.0, 2.0);
    let pane_state = state.panes.get(pane).expect("pane");
    assert_eq!(pane_state.editor.viewport.first_line, expected);
    assert_eq!(
        pane_state.editor.document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert!(!pane_state.editor.viewport_tracks_cursor);
    assert_eq!(state.status_message, "viewport scrolled");
}

fn replacement_key_event(
    key: Key,
    modifiers: keyboard::Modifiers,
    text: Option<&str>,
) -> keyboard::Event {
    keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers,
        text: text.map(Into::into),
        repeat: false,
    }
}

#[test]
fn gui_editor_replacement_keyboard_bridge_maps_text_and_navigation() {
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("a".into()),
            keyboard::Modifiers::NONE,
            Some("a"),
        )),
        vec![GuiEditorReplacementInput::InsertChar('a')]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("A".into()),
            keyboard::Modifiers::SHIFT,
            Some("A"),
        )),
        vec![GuiEditorReplacementInput::InsertChar('A')]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("x".into()),
            keyboard::Modifiers::CTRL,
            Some("x"),
        )),
        Vec::<GuiEditorReplacementInput>::new()
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("a".into()),
            keyboard::Modifiers::CTRL,
            Some("a"),
        )),
        vec![GuiEditorReplacementInput::SelectAll]
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("c".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Copy)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("x".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Cut)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("v".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Paste)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("z".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Undo)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("z".into()),
            keyboard::Modifiers::CTRL.union(keyboard::Modifiers::SHIFT),
            None,
        )),
        Some(GuiMenuCommand::Redo)
    );
    assert_eq!(
        gui_editor_clipboard_shortcut_command(&replacement_key_event(
            Key::Character("y".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        Some(GuiMenuCommand::Redo)
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Enter),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::InsertNewline]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Backspace),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteBackward]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Delete),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteForward]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Escape),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::ClearSelection]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Home),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::MoveLineStart]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::End),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::MoveLineEnd]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowDown),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Down)]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::PageDown),
            keyboard::Modifiers::NONE,
            None,
        )),
        vec![GuiEditorReplacementInput::ScrollViewportLines(
            GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES as i32
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowLeft),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(
            kfnotepad::CursorMove::WordLeft
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::ArrowRight),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::Move(
            kfnotepad::CursorMove::WordRight
        )]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Backspace),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeletePreviousWord]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Named(Named::Delete),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteNextWord]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_keyboard_event(&replacement_key_event(
            Key::Character("k".into()),
            keyboard::Modifiers::CTRL,
            None,
        )),
        vec![GuiEditorReplacementInput::DeleteToLineEnd]
    );
}

#[test]
fn gui_editor_replacement_ime_commit_maps_to_text_insertion() {
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
            "かな".to_string()
        )),
        vec![
            GuiEditorReplacementInput::InsertChar('か'),
            GuiEditorReplacementInput::InsertChar('な'),
        ]
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Preedit(
            "か".to_string(),
            Some(0..3),
        )),
        Vec::<GuiEditorReplacementInput>::new()
    );
    assert_eq!(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Closed),
        Vec::<GuiEditorReplacementInput>::new()
    );
}

#[test]
fn gui_editor_ime_preedit_renders_at_cursor_without_mutating_line() {
    let line = GuiEditorViewportLine {
        number: 1,
        text: "start end".to_string(),
        cursor_column: Some(6),
        selection: None,
        syntax_segments: Some(vec![GuiEditorSyntaxSegment {
            text: "start end".to_string(),
            color: Color::WHITE,
        }]),
    };
    let preedit = GuiImePreedit {
        tile_id: GuiTileId(1),
        content: "かな".to_string(),
        selection: Some(0..3),
    };

    let rendered = gui_editor_viewport_line_with_ime_preedit(line.clone(), Some(&preedit));

    assert_eq!(line.text, "start end");
    assert_eq!(rendered.text, "start かなend");
    assert_eq!(rendered.cursor_column, None);
    assert_eq!(
        rendered.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 6,
            end_column: 7,
        })
    );
    assert_eq!(rendered.syntax_segments, None);
}

#[test]
fn gui_editor_ime_preedit_selection_expands_to_grapheme_boundaries() {
    let preedit = GuiImePreedit {
        tile_id: GuiTileId(1),
        content: "e\u{301}x".to_string(),
        selection: Some(1..3),
    };

    assert_eq!(gui_ime_preedit_selection_columns(&preedit), Some((0, 2)));

    let line = GuiEditorViewportLine {
        number: 1,
        text: "ab".to_string(),
        cursor_column: Some(1),
        selection: None,
        syntax_segments: None,
    };
    let rendered = gui_editor_viewport_line_with_ime_preedit(line, Some(&preedit));

    assert_eq!(rendered.text, "ae\u{301}xb");
    assert_eq!(
        rendered.selection,
        Some(GuiEditorSelectionSpan {
            start_column: 1,
            end_column: 3,
        })
    );
}

#[test]
fn gui_editor_ime_request_cursor_rect_tracks_visual_row_and_gutter() {
    let request = GuiImeInputMethodRequest {
        visual_row: 3,
        cursor_column: 5,
        gutter_width: 42.0,
        character_width: 9.0,
        row_height: 18.0,
        preedit: Some(input_method::Preedit {
            content: "かな".to_string(),
            selection: Some(0..3),
            text_size: Some(Pixels(16.0)),
        }),
    };

    assert_eq!(
        request.cursor_rect(Rectangle::new(
            iced::Point::new(10.0, 20.0),
            Size::new(500.0, 300.0)
        )),
        Rectangle::new(iced::Point::new(97.0, 74.0), Size::new(1.0, 18.0))
    );
    assert_eq!(
        request
            .preedit
            .as_ref()
            .map(|preedit| preedit.content.as_str()),
        Some("かな")
    );
}

#[test]
fn gui_editor_replacement_keyboard_bridge_applies_when_explicitly_routed() {
    let temp = TempArea::new("gui-replacement-bridge");
    let file = temp.path("bridge.txt");
    fs::write(&file, "one\ntwo").expect("write bridge");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::InsertChar('X'),
        GuiEditorReplacementInput::Move(kfnotepad::CursorMove::Right),
        GuiEditorReplacementInput::InsertNewline,
    ]);

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "Xo\nne\ntwo"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.active_editor().text(), "Xo\nne\ntwo");
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.status_message, "replacement edit");
}

#[test]
fn gui_editor_replacement_home_end_move_within_current_line() {
    let temp = TempArea::new("gui-replacement-home-end");
    let file = temp.path("home-end.txt");
    fs::write(&file, "abc\ndefgh\n").expect("write home end");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 1, column: 2 });
    state.sync_active_editor_to_document();

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::MoveLineEnd,
    ]);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 5 }
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::MoveLineStart,
    ]);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "abc\ndefgh\n"
    );
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
}

#[test]
fn gui_editor_replacement_ime_commit_applies_when_explicitly_routed() {
    let temp = TempArea::new("gui-replacement-ime");
    let file = temp.path("ime.txt");
    fs::write(&file, "start").expect("write ime");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(
        gui_editor_replacement_inputs_from_ime_event(&input_method::Event::Commit(
            "かな".to_string(),
        )),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "かなstart"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 2 }
    );
    assert_eq!(state.active_editor().text(), "かなstart");
}

#[test]
fn gui_editor_replacement_ime_preedit_is_transient_until_commit() {
    let temp = TempArea::new("gui-replacement-ime-preedit");
    let file = temp.path("ime-preedit.txt");
    fs::write(&file, "start").expect("write ime preedit");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;

    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Opened),
    );
    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Preedit("かな".to_string(), Some(0..3))),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "start"
    );
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        state.replacement_ime_preedit,
        Some(GuiImePreedit {
            tile_id,
            content: "かな".to_string(),
            selection: Some(0..3),
        })
    );

    let _ = update(
        &mut state,
        Message::ReplacementEditorIme(input_method::Event::Commit("かな".to_string())),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "かなstart"
    );
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(state.replacement_ime_preedit, None);
}

#[test]
fn gui_editor_replacement_selection_persists_until_next_active_tile_edit() {
    let temp = TempArea::new("gui-replacement-selection");
    let file = temp.path("selection.txt");
    fs::write(&file, "alpha beta").expect("write selection");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::SelectRange {
            anchor: DocumentCursor { row: 0, column: 6 },
            focus: DocumentCursor { row: 0, column: 10 },
        },
    ]);

    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .and_then(|pane| pane.editor.replacement_selection),
        Some(GuiEditorReplacementSelection {
            anchor: DocumentCursor { row: 0, column: 6 },
            focus: DocumentCursor { row: 0, column: 10 },
        })
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 10 }
    );
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "alpha beta"
    );

    state.apply_replacement_editor_inputs_to_active_tile(vec![
        GuiEditorReplacementInput::InsertChar('X'),
    ]);

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "alpha X"
    );
    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .and_then(|pane| pane.editor.replacement_selection),
        None
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 7 }
    );
    assert_eq!(state.active_editor().text(), "alpha X");
}

#[test]
fn gui_editor_replacement_message_edits_active_tile_when_renderer_is_live() {
    let temp = TempArea::new("gui-replacement-live");
    let file = temp.path("live.txt");
    fs::write(&file, "unchanged").expect("write live");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "Xunchanged"
    );
    assert_eq!(state.active_editor().text(), "Xunchanged");
    assert_eq!(state.status_message, "replacement edit");
}

#[test]
fn gui_restore_last_workspace_toggle_persists_config() {
    let temp = TempArea::new("gui-restore-toggle");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

    assert!(state.settings.gui_restore_last_workspace);
    assert_eq!(state.status_message, "restore last workspace: on");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(false));

    assert!(!state.settings.gui_restore_last_workspace);
    assert_eq!(state.status_message, "restore last workspace: off");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
fn gui_restore_last_workspace_toggle_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-restore-toggle-failure");
    let blocked_parent = temp.path("blocked");
    fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(blocked_parent.join("config.toml"));
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

    assert!(!state.settings.gui_restore_last_workspace);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}
