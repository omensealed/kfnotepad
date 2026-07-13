use super::*;

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
