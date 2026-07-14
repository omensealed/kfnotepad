use super::*;

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
    #[cfg(feature = "syntax")]
    assert!(viewport_without_syntax.lines[0].syntax_segments.is_some());
    #[cfg(not(feature = "syntax"))]
    assert!(viewport_without_syntax.lines[0].syntax_segments.is_none());
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
#[cfg(feature = "syntax")]
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

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    adapter.apply(GuiEditorCommand::Paste("alpha".to_string()));
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
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
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    adapter.apply(GuiEditorCommand::Delete);
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(adapter.text(), "lpha");
}

#[test]
fn gui_editor_adapter_materializes_reverse_multiline_selection_without_reconstruction() {
    let anchor = DocumentCursor { row: 1, column: 3 };
    let focus = DocumentCursor { row: 0, column: 1 };
    let mut paste_adapter = GuiEditorAdapter::from_text("one\ntwo\nthird");
    paste_adapter.set_replacement_selection(anchor, focus, focus);

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    paste_adapter.apply(GuiEditorCommand::Paste("X".to_string()));

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(paste_adapter.text(), "oX\nthird");
    assert_eq!(paste_adapter.replacement_selection, None);

    let mut delete_adapter = GuiEditorAdapter::from_text("one\ntwo\nthird");
    delete_adapter.set_replacement_selection(anchor, focus, focus);
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    delete_adapter.apply(GuiEditorCommand::Delete);

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(delete_adapter.text(), "o\nthird");
    assert_eq!(delete_adapter.replacement_selection, None);

    let mut full_delete_adapter = GuiEditorAdapter::from_text("alpha\n");
    full_delete_adapter.apply(GuiEditorCommand::SelectAll);
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    full_delete_adapter.apply(GuiEditorCommand::Delete);

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(full_delete_adapter.text(), "");
    assert_eq!(full_delete_adapter.replacement_selection, None);
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
fn gui_editor_adapter_page_motion_does_not_reconstruct_a_text_buffer() {
    let mut adapter = GuiEditorAdapter::from_text(&numbered_lines(100));

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    adapter.apply(GuiEditorCommand::IcedAction(text_editor::Action::Move(
        text_editor::Motion::PageDown,
    )));

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(
        adapter.document_cursor(),
        DocumentCursor {
            row: GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES,
            column: 0,
        }
    );
}
