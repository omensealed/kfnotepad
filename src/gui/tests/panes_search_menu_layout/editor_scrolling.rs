use super::*;

#[test]
fn gui_viewport_scroll_message_routes_to_active_editor() {
    let temp = TempArea::new("gui-viewport-scroll-message");
    let file = temp.path("viewport.txt");
    let text = numbered_lines(100);
    fs::write(&file, &text).expect("write viewport");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::ScrollActiveEditorViewport(2));

    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 2, column: 0 }
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 2, column: 0 }
    );
    assert_eq!(
        state
            .active_editor()
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 3,
            text: gui_line_number_gutter_text(3, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );
    assert_eq!(state.status_message, "viewport down");

    let _ = update(&mut state, Message::ScrollActiveEditorViewport(-99));

    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 2, column: 0 }
    );
    assert_eq!(
        state
            .active_editor()
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 1,
            text: gui_line_number_gutter_text(1, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );
    assert_eq!(state.status_message, "viewport up");
}

#[test]
fn gui_iced_scroll_action_keeps_gutter_synced_without_dirtying_document() {
    let temp = TempArea::new("gui-iced-scroll-gutter-sync");
    let file = temp.path("iced-scroll.txt");
    let text = numbered_lines(100);
    fs::write(&file, &text).expect("write Iced scroll action fixture");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;
    let tile_id = state.workspace.active_tile().id;

    let _ = update(
        &mut state,
        Message::Edit(pane, text_editor::Action::Scroll { lines: 5 }),
    );

    assert!(!state
        .workspace
        .tile(tile_id)
        .expect("tile")
        .document
        .buffer
        .is_dirty());
    assert_eq!(
        state
            .active_editor()
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 6,
            text: gui_line_number_gutter_text(6, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );
    assert_eq!(state.status_message, "scrolled");
}

#[test]
fn gui_replacement_editor_wheel_delta_maps_to_viewport_lines() {
    let settings = EditorSettings {
        gui_font_size: 20,
        ..EditorSettings::default()
    };

    assert_eq!(
        gui_editor_replacement_scroll_delta_lines(
            mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
            settings,
        ),
        3
    );
    assert_eq!(
        gui_editor_replacement_scroll_delta_lines(
            mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 },
            settings,
        ),
        -2
    );
    assert_eq!(
        gui_editor_replacement_scroll_delta_lines(
            mouse::ScrollDelta::Pixels {
                x: 0.0,
                y: -(20.0 * GUI_EDITOR_LINE_HEIGHT * 2.0),
            },
            settings,
        ),
        2
    );
}

#[test]
fn gui_replacement_editor_wheel_scrolls_tile_without_dirtying_document() {
    let temp = TempArea::new("gui-replacement-wheel-scroll");
    let file = temp.path("replacement-wheel.txt");
    let text = numbered_lines(100);
    fs::write(&file, &text).expect("write wheel file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    let pane = state.active_pane;
    let tile_id = state.workspace.active_tile().id;

    let _ = update(&mut state, Message::ReplacementEditorWheelScrolled(pane, 4));

    let tile = state.workspace.tile(tile_id).expect("tile");
    assert!(!tile.document.buffer.is_dirty());
    assert_eq!(tile.state.cursor, DocumentCursor { row: 0, column: 0 });
    assert_eq!(
        state
            .panes
            .get(pane)
            .expect("pane")
            .editor
            .document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(
        state
            .panes
            .get(pane)
            .expect("pane")
            .editor
            .render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 16)
            .line_numbers,
        GuiEditorLineNumberSnapshot {
            line_count: 100,
            gutter_start: 5,
            text: gui_line_number_gutter_text(5, 100, GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES),
            width: gui_line_number_gutter_width(100, 16),
        }
    );
    assert_eq!(state.status_message, "viewport down");
}
