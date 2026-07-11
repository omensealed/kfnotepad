#[test]
fn gui_browser_clicks_are_ignored_while_workspace_panel_is_active() {
    let temp = TempArea::new("gui-left-panel-ignore-browser");
    let file = temp.path("from-browser.txt");
    fs::write(&file, "browser file\n").expect("write browser file");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let initial_path = state.workspace.active_tile().document.path.clone();
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "from-browser.txt")
        .expect("browser file entry");

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
    );
    state.activate_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial_path);
    assert_ne!(state.workspace.active_tile().document.path, file);
}

#[test]
fn gui_browser_directory_single_click_selects_without_navigation() {
    let temp = TempArea::new("gui-browser-nav");
    fs::create_dir(temp.path("subdir")).expect("create subdir");
    fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "subdir/")
        .expect("subdir entry");

    state.select_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(
        state.browser.as_ref().expect("browser").sidebar.current_dir,
        temp.root.canonicalize().expect("canonical root")
    );
    assert_eq!(
        state
            .browser
            .as_ref()
            .expect("browser")
            .selected_entry()
            .expect("selected")
            .path,
        temp.path("subdir")
    );
}
#[test]
fn gui_browser_directory_double_click_navigates_without_opening_pane() {
    let temp = TempArea::new("gui-browser-nav-double");
    fs::create_dir(temp.path("subdir")).expect("create subdir");
    fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "subdir/")
        .expect("subdir entry");

    state.activate_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(
        state.browser.as_ref().expect("browser").sidebar.current_dir,
        temp.path("subdir")
            .canonicalize()
            .expect("canonical subdir")
    );
    assert!(state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "inside.txt"));
}

#[test]
fn gui_browser_toggle_hides_and_restores_open_behavior() {
    let temp = TempArea::new("gui-browser-toggle");
    let file = temp.path("visible-again.txt");
    fs::write(&file, "visible\n").expect("write file");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "visible-again.txt")
        .expect("browser file entry");

    let _ = update(&mut state, Message::ToggleBrowser);
    assert!(!state.browser_visible);
    state.activate_browser_entry(index);
    assert_eq!(state.workspace.tiles.len(), 1);

    let _ = update(&mut state, Message::ToggleBrowser);
    assert!(state.browser_visible);
    state.select_browser_entry(index);
    assert_ne!(state.workspace.active_tile().document.path, file);
    state.activate_browser_entry(index);
    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
}

#[test]
fn gui_close_active_clean_pane_removes_tile_and_focuses_fallback() {
    let temp = TempArea::new("gui-close-clean");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert!(state.status_message.starts_with("closed "));
}

#[test]
fn gui_close_dirty_pane_requires_second_request() {
    let temp = TempArea::new("gui-close-dirty");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("discard me\n");

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert!(state
        .status_message
        .contains("unsaved changes; close again"));

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert_eq!(
        fs::read_to_string(&second).expect("read second"),
        "second\n"
    );
}

#[test]
fn gui_window_close_clean_state_allows_close_without_prompt() {
    let temp = TempArea::new("gui-window-close-clean");
    let file = temp.path("clean.txt");
    fs::write(&file, "clean\n").expect("write clean");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });

    let _task = update(
        &mut state,
        Message::WindowCloseRequested(window::Id::unique()),
    );

    assert!(!state.pending_app_quit);
    assert!(!state
        .status_message
        .contains("close window again to discard"));
}

#[test]
fn gui_window_close_dirty_state_requires_second_request_without_saving() {
    let temp = TempArea::new("gui-window-close-dirty");
    let file = temp.path("dirty.txt");
    fs::write(&file, "original\n").expect("write dirty");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("discard from app close\n");

    let _task = update(
        &mut state,
        Message::WindowCloseRequested(window::Id::unique()),
    );

    assert!(state.pending_app_quit);
    assert_eq!(
        state.status_message,
        "unsaved changes; close window again to discard all dirty tiles"
    );
    assert_eq!(
        fs::read_to_string(&file).expect("read original"),
        "original\n"
    );

    let _task = update(
        &mut state,
        Message::WindowCloseRequested(window::Id::unique()),
    );

    assert!(state.pending_app_quit);
    assert_eq!(
        fs::read_to_string(&file).expect("read original after confirm"),
        "original\n"
    );
}

#[test]
fn gui_ctrl_q_quit_uses_window_close_dirty_confirmation() {
    let temp = TempArea::new("gui-ctrl-q-dirty");
    let file = temp.path("dirty.txt");
    fs::write(&file, "original\n").expect("write dirty");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("dirty from ctrl-q\n");

    let _task = update(&mut state, Message::QuitRequested(window::Id::unique()));

    assert!(state.pending_app_quit);
    assert_eq!(
        state.status_message,
        "unsaved changes; close window again to discard all dirty tiles"
    );
    assert_eq!(
        fs::read_to_string(&file).expect("read original"),
        "original\n"
    );
}

#[test]
fn gui_window_close_pending_confirmation_clears_after_save() {
    let temp = TempArea::new("gui-window-close-save-clears");
    let file = temp.path("dirty.txt");
    fs::write(&file, "original\n").expect("write dirty");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved before close\n");

    let _task = update(
        &mut state,
        Message::WindowCloseRequested(window::Id::unique()),
    );
    assert!(state.pending_app_quit);
    let tile_id = state.workspace.active_tile().id;
    let _ = update(&mut state, Message::SaveRequested);
    let source_revision = state.workspace.active_tile().document.buffer.edit_revision();
    fs::write(&file, "saved before close\n").expect("simulate async save");
    let snapshot = gui_file_snapshot(&file)
        .expect("snapshot saved file")
        .expect("saved file");
    let _ = update(
        &mut state,
        Message::SaveActiveTileCompleted {
            tile_id,
            result: Ok(GuiSaveResult {
                source_revision,
                snapshot,
            }),
        },
    );

    assert!(!state.pending_app_quit);
    assert_eq!(
        fs::read_to_string(&file).expect("read saved"),
        "saved before close\n"
    );
}

#[test]
fn gui_save_async_completion_keeps_dirty_when_text_changed_before_finish() {
    let temp = TempArea::new("gui-window-close-save-while-editing");
    let path = temp.path("dirty.txt");
    fs::write(&path, "original\n").expect("write dirty");

    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });
    let tile_id = state.workspace.active_tile().id;
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("queued save text\n");

    let _task = update(&mut state, Message::SaveRequested);
    let source_revision = state.workspace.active_tile().document.buffer.edit_revision();
    fs::write(&path, "queued save text\n").expect("simulate async save");
    let snapshot = gui_file_snapshot(&path)
        .expect("snapshot queued save")
        .expect("queued save file");

    state
        .workspace
        .active_tile_mut()
        .document
        .buffer
        .replace_text("changed after queued save\n");

    let _ = update(
        &mut state,
        Message::SaveActiveTileCompleted {
            tile_id,
            result: Ok(GuiSaveResult {
                source_revision,
                snapshot,
            }),
        },
    );

    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert_eq!(
        fs::read_to_string(&path).expect("read queued save"),
        "queued save text\n"
    );
    assert_eq!(
        state.status_message,
        "save completed after edits; save again to persist latest text"
    );
}

#[test]
fn gui_repeated_save_requests_coalesce_to_one_follow_up_per_tile() {
    let temp = TempArea::new("gui-save-coalescing");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved\n");
    let tile_id = state.workspace.active_tile().id;

    let _first = update(&mut state, Message::SaveRequested);
    let source_revision = state.workspace.active_tile().document.buffer.edit_revision();
    let _second = update(&mut state, Message::SaveRequested);
    let _third = update(&mut state, Message::SaveRequested);

    assert!(state.save_in_flight.contains(&tile_id));
    assert!(state.save_requested_after_in_flight.contains(&tile_id));
    assert_eq!(state.save_in_flight.len(), 1);

    fs::write(&path, "saved\n").expect("simulate first save");
    let snapshot = gui_file_snapshot(&path)
        .expect("snapshot first save")
        .expect("first saved file");
    let _follow_up = update(
        &mut state,
        Message::SaveActiveTileCompleted {
            tile_id,
            result: Ok(GuiSaveResult {
                source_revision,
                snapshot,
            }),
        },
    );

    assert!(state.save_in_flight.contains(&tile_id));
    assert!(!state.save_requested_after_in_flight.contains(&tile_id));

    let follow_up_revision = state.workspace.active_tile().document.buffer.edit_revision();
    let snapshot = gui_file_snapshot(&path)
        .expect("snapshot follow-up")
        .expect("follow-up file");
    let _ = update(
        &mut state,
        Message::SaveActiveTileCompleted {
            tile_id,
            result: Ok(GuiSaveResult {
                source_revision: follow_up_revision,
                snapshot,
            }),
        },
    );
    assert!(!state.save_in_flight.contains(&tile_id));
}

#[test]
fn gui_search_next_and_previous_update_shared_and_editor_cursor() {
    let temp = TempArea::new("gui-search-next-prev");
    let file = temp.path("search.txt");
    fs::write(&file, "alpha\nbeta alpha\n").expect("write search");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });

    let _ = update(&mut state, Message::SearchQueryChanged("alpha".to_string()));
    let _ = update(&mut state, Message::SearchNext);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 5 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 1, column: 5 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("alpha"));
    assert_eq!(state.status_message, "found next: alpha");
    let highlight = state.search_highlight.as_ref().expect("search highlight");
    assert_eq!(highlight.tile_id, state.workspace.active_tile().id);
    assert_eq!(highlight.query, "alpha");

    let _ = update(&mut state, Message::SearchPrevious);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("alpha"));
    assert_eq!(state.status_message, "found previous: alpha");
    assert_eq!(
        state
            .search_highlight
            .as_ref()
            .map(|highlight| highlight.query.as_str()),
        Some("alpha")
    );

    let _ = update(&mut state, Message::SearchQueryChanged("beta".to_string()));
    assert_eq!(state.search_highlight, None);
}

#[test]
fn gui_find_history_keeps_ten_recent_unique_queries() {
    let temp = TempArea::new("gui-find-history");
    let file = temp.path("search.txt");
    fs::write(&file, "alpha beta gamma delta epsilon\n").expect("write search file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    for query in [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
        "lambda", "beta",
    ] {
        let _ = update(&mut state, Message::SearchQueryChanged(query.to_string()));
        let _ = update(&mut state, Message::SearchNext);
    }

    assert_eq!(state.search_history.len(), GUI_FIND_HISTORY_LIMIT);
    assert_eq!(
        state.search_history.first().map(String::as_str),
        Some("beta")
    );
    assert_eq!(
        state
            .search_history
            .iter()
            .filter(|entry| entry.as_str() == "beta")
            .count(),
        1
    );
    assert!(!state.search_history.iter().any(|entry| entry == "alpha"));

    let _ = update(&mut state, Message::SearchQueryChanged(String::new()));
    assert!(state.search_history_open);

    let _ = update(
        &mut state,
        Message::SearchHistorySelected("theta".to_string()),
    );
    assert_eq!(state.search_query, "theta");
    assert_eq!(
        state.search_history.first().map(String::as_str),
        Some("theta")
    );
    assert!(!state.search_history_open);
}

#[test]
fn gui_search_reports_missing_query_and_no_match() {
    let temp = TempArea::new("gui-search-status");
    let file = temp.path("search.txt");
    fs::write(&file, "alpha\n").expect("write search");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });

    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "search query required");

    let _ = update(
        &mut state,
        Message::SearchQueryChanged("missing".to_string()),
    );
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "no match: missing");
    assert_eq!(state.search_highlight, None);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
}

#[test]
fn gui_document_edge_navigation_updates_editor_cursor() {
    let temp = TempArea::new("gui-edge-navigation");
    let file = temp.path("nav.txt");
    fs::write(&file, "one\ntwo\nthree").expect("write nav");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });

    let _ = update(&mut state, Message::GoDocumentEnd);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 2, column: 5 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 2, column: 5 }
    );
    assert_eq!(state.status_message, "moved to document end");

    let _ = update(&mut state, Message::GoDocumentStart);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(state.status_message, "moved to document start");
}

#[test]
fn gui_go_to_line_updates_shared_and_editor_cursor() {
    let temp = TempArea::new("gui-go-to-line");
    let file = temp.path("goto.txt");
    fs::write(&file, "one\ntwo words\nthree\n").expect("write goto");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 99 });

    let _ = update(&mut state, Message::GoToLineQueryChanged("2".to_string()));
    let _ = update(&mut state, Message::GoToLineRequested);

    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 9 }
    );
    assert_eq!(
        state.active_editor().document_cursor(),
        DocumentCursor { row: 1, column: 9 }
    );
    assert_eq!(state.status_message, "Line 2");
}

#[test]
fn gui_go_to_line_reports_validation_without_moving_cursor() {
    let temp = TempArea::new("gui-go-to-line-validation");
    let file = temp.path("goto.txt");
    fs::write(&file, "one\ntwo\n").expect("write goto");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 1 });

    for (query, expected_status) in [
        ("", "Line number is empty"),
        ("abc", "Line number is invalid"),
        ("99", "Line out of range: 99"),
    ] {
        let _ = update(&mut state, Message::GoToLineQueryChanged(query.to_string()));
        let _ = update(&mut state, Message::GoToLineRequested);
        assert_eq!(state.status_message, expected_status);
        assert_eq!(
            state.workspace.active_tile().state.cursor,
            DocumentCursor { row: 0, column: 1 }
        );
    }
}

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
fn gui_native_editor_scroll_keeps_gutter_synced_without_dirtying_document() {
    let temp = TempArea::new("gui-native-scroll-gutter-sync");
    let file = temp.path("native-scroll.txt");
    let text = numbered_lines(100);
    fs::write(&file, &text).expect("write native scroll");
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

#[test]
fn gui_menu_surface_lists_primary_command_groups() {
    assert!(gui_menu_uses_iced_aw_menu_tree());
    assert_eq!(
            gui_menu_submenu_policy(),
            "Keep current root command groups flat until a group gains enough depth to justify nested hover submenus."
        );
    assert_eq!(
        gui_menu_groups().map(gui_menu_group_label),
        ["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
    );
    assert_eq!(
        gui_menu_groups()
            .map(gui_menu_group_chrome_label)
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
    );
    assert_eq!(
        gui_menu_dropdown_labels(GuiMenuGroup::File),
        vec![
            LABEL_NEW_TILE,
            LABEL_OPEN,
            LABEL_OPEN_PATH,
            LABEL_SAVE,
            LABEL_SAVE_AS,
            LABEL_SAVE_AS_PATH,
            LABEL_CLOSE_TILE,
            LABEL_QUIT,
        ]
    );
    assert_eq!(
        gui_menu_dropdown_labels(GuiMenuGroup::Edit),
        vec![
            LABEL_UNDO,
            LABEL_REDO,
            LABEL_COPY,
            LABEL_CUT,
            LABEL_PASTE,
            LABEL_SELECT_ALL,
            LABEL_FIND_NEXT,
            LABEL_FIND_PREVIOUS,
        ]
    );
    assert_eq!(gui_menu_group_index(GuiMenuGroup::File), 0);
    assert_eq!(gui_menu_group_index(GuiMenuGroup::Tile), 5);
    assert_eq!(gui_menu_group_index(GuiMenuGroup::Help), 6);

    let file_commands: Vec<_> = gui_menu_items(GuiMenuGroup::File)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        file_commands,
        vec![
            GuiMenuCommand::NewTile,
            GuiMenuCommand::Open,
            GuiMenuCommand::OpenPath,
            GuiMenuCommand::Save,
            GuiMenuCommand::SaveAs,
            GuiMenuCommand::SaveAsPath,
            GuiMenuCommand::ClosePane,
            GuiMenuCommand::Quit,
        ]
    );

    let edit_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Edit)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        edit_commands,
        vec![
            GuiMenuCommand::Undo,
            GuiMenuCommand::Redo,
            GuiMenuCommand::Copy,
            GuiMenuCommand::Cut,
            GuiMenuCommand::Paste,
            GuiMenuCommand::SelectAll,
            GuiMenuCommand::FindNext,
            GuiMenuCommand::FindPrevious,
        ]
    );

    let go_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Go)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        go_commands,
        vec![
            GuiMenuCommand::GoToLine,
            GuiMenuCommand::GoDocumentStart,
            GuiMenuCommand::GoDocumentEnd,
        ]
    );

    let notes_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Notes)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        notes_commands,
        vec![
            GuiMenuCommand::OpenManagedNote,
            GuiMenuCommand::ListManagedNotes,
        ]
    );

    let tile_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Tile)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        tile_commands,
        vec![
            GuiMenuCommand::ToggleMinimize,
            GuiMenuCommand::ToggleMaximize,
            GuiMenuCommand::EqualizeTiles,
            GuiMenuCommand::MoveLeft,
            GuiMenuCommand::MoveRight,
            GuiMenuCommand::MoveUp,
            GuiMenuCommand::MoveDown,
        ]
    );

    let help_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Help)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(help_commands, vec![GuiMenuCommand::OpenHelp]);
}

#[test]
fn gui_help_menu_opens_builtin_help_document_tile() {
    let temp = TempArea::new("gui-help-menu");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));

    let active = state.workspace.active_tile();
    assert_eq!(active.document.path, temp.path(GUI_HELP_DOCUMENT_PATH));
    let help_text = active.document.buffer.to_text();
    assert!(help_text.contains("# kfnotepad help"));
    assert!(help_text.contains("Double-click a file to open it in a tile."));
    assert!(help_text.contains("Ctrl-R, View > Reader mode"));
    assert!(help_text.contains("Search is case-insensitive by default."));
    assert!(help_text.contains("Ctrl-Shift-T cycles the syntax highlighting theme"));
    assert!(help_text.contains("Reader speed is configured in Preferences"));
    assert!(help_text.contains("Notes > Open note creates or opens a managed Markdown note."));
    assert!(help_text.contains("Tile > Equalize tiles arranges open tiles into an even grid."));
    assert!(help_text.contains("Opening a file that is already open focuses or restores"));
    assert!(help_text.contains("Save uses the same atomic local-file adapter"));
    assert!(!active.document.buffer.is_dirty());
    assert!(state.status_message.contains("opened help"));

    let tile_count = state.workspace.tiles.len();
    let help_path = active.document.path.clone();
    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));
    assert_eq!(state.workspace.tiles.len(), tile_count);
    assert_eq!(state.workspace.active_tile().document.path, help_path);
}

#[test]
fn gui_menu_styles_use_app_theme_palette() {
    let palette = gui_theme_palette(EditorThemeId::Nocturne);
    let panel_style = gui_menu_panel_style(palette);

    assert_eq!(
        panel_style.menu_border.radius.top_left,
        GUI_MENU_DROPDOWN_RADIUS
    );
    assert_eq!(panel_style.menu_border.color, palette.primary);
    assert_eq!(panel_style.menu_shadow.offset, Vector::new(0.0, 6.0));
    assert_eq!(panel_style.menu_shadow.blur_radius, 16.0);
    assert_eq!(
        panel_style.path_border.radius.top_left,
        GUI_MENU_ITEM_RADIUS
    );

    let active = gui_menu_item_button_style(palette, iced::widget::button::Status::Active);
    assert_eq!(active.text_color, palette.text);
    assert_eq!(active.border.width, 0.0);
    assert_eq!(active.border.radius.top_left, GUI_MENU_ITEM_RADIUS);

    let root = gui_menu_root_style(palette);
    assert_eq!(root.text_color, Some(palette.text));
    assert!(root.background.is_none());
    assert_eq!(GUI_MENU_ROOT_HORIZONTAL_PADDING, 3.0);
    assert_eq!(GUI_MENU_ROOT_VERTICAL_PADDING, 1.0);
    assert_eq!(GUI_MENU_ROOT_HEIGHT, 24.0);
    assert_eq!(GUI_MENU_BAR_SPACING, 1);
    assert_eq!(GUI_HEADER_ACTION_SPACING, 3);
    assert_eq!(GUI_HEADER_GROUP_SPACING, 6);
    assert_eq!(GUI_HEADER_SPLIT_SPACING, 3);
    assert_eq!(GUI_MENU_ITEM_PADDING, [3, 5]);
    assert_eq!(gui_icon_font(), iced_fonts::NERD_FONT);
    assert_eq!(GUI_ICON_LINE_HEIGHT, 1.0);
    assert_eq!(GUI_PANEL_PADDING_LEFT, 2.0);
    assert_eq!(GUI_PANEL_PADDING_RIGHT, 4.0);
    assert_eq!(GUI_PANEL_PADDING_VERTICAL, 6.0);
    assert_eq!(GUI_EDITOR_RENDER_LINE_BUDGET, 512);

    let hovered = gui_menu_item_button_style(palette, iced::widget::button::Status::Hovered);
    assert_eq!(hovered.text_color, palette.background);
    assert_eq!(hovered.border.width, 1.0);
    assert_eq!(hovered.border.color, palette.primary);

    let chrome = gui_chrome_button_style(palette, iced::widget::button::Status::Active);
    assert_eq!(chrome.text_color, palette.background);
    assert_eq!(chrome.border.width, 0.0);
    assert_eq!(chrome.border.radius.top_left, 4.0);
}

#[test]
fn gui_native_editor_and_form_styles_do_not_add_hover_borders() {
    let palette = gui_theme_palette(EditorThemeId::Nocturne);

    let active = gui_native_editor_style(palette, text_editor::Status::Active, false);
    let hovered = gui_native_editor_style(palette, text_editor::Status::Hovered, false);
    let focused = gui_native_editor_style(
        palette,
        text_editor::Status::Focused { is_hovered: true },
        false,
    );
    let search = gui_native_editor_style(palette, text_editor::Status::Active, true);
    assert_eq!(hovered.border, active.border);
    assert_eq!(focused.border, active.border);
    assert_eq!(active.border.width, 0.0);
    assert_eq!(active.border.color, Color::TRANSPARENT);
    assert!(search.selection.a > active.selection.a);
    assert_eq!(search.selection.r, active.selection.r);
    assert_eq!(search.selection.g, active.selection.g);
    assert_eq!(search.selection.b, active.selection.b);

    let input_active = gui_text_input_style(palette, iced::widget::text_input::Status::Active);
    let input_hovered = gui_text_input_style(palette, iced::widget::text_input::Status::Hovered);
    assert_eq!(input_hovered.border, input_active.border);
    assert_eq!(input_active.value, palette.text);

    let checkbox_active = gui_checkbox_style(
        palette,
        iced::widget::checkbox::Status::Active { is_checked: true },
    );
    assert_eq!(checkbox_active.text_color, Some(palette.text));
    assert_eq!(checkbox_active.border.color, palette.primary);
}

#[test]
fn gui_chrome_labels_trim_paths_and_keep_tooltip_sources() {
    let path = PathBuf::from("/tmp/kfnotepad/deep/example.md");

    assert_eq!(gui_file_name_label(&path), "example.md");
    assert_eq!(
        gui_tile_title_label(&path, true, "modified"),
        "active | example.md | modified"
    );
    assert_eq!(
        gui_tile_title_label(&path, true, "saved"),
        "active | example.md"
    );
    assert_eq!(gui_tile_title_label(&path, false, "saved"), "example.md");
    assert_eq!(
        gui_icon_label(ICON_FILES, LABEL_FILES),
        format!("{ICON_FILES} Files")
    );
    assert_eq!(gui_icon_only_label(ICON_FILES), ICON_FILES);
    assert!(!gui_icon_only_label(ICON_PREFERENCES).contains(LABEL_PREFERENCES));
    assert_eq!(gui_icon_only_label(ICON_NEW_TILE), ICON_NEW_TILE);
    assert!(!gui_icon_only_label(ICON_NEW_TILE).contains(LABEL_NEW_TILE));
    assert_eq!(gui_icon_only_label(ICON_SAVE), ICON_SAVE);
    assert!(!gui_icon_only_label(ICON_SAVE).contains(LABEL_SAVE));
    assert_eq!(gui_icon_only_label(ICON_THEME), ICON_THEME);
    assert!(!gui_icon_only_label(ICON_THEME).contains(LABEL_THEME));
    assert!(!gui_tile_title_label(&path, true, "modified").contains("/tmp"));
    assert!(gui_tile_title_controls_attached(true));
    assert!(gui_tile_title_controls_attached(false));

    let deep_path = PathBuf::from("/home/example/projects/kfnotepad/docs");
    assert_eq!(
        gui_sidebar_path_label(&path),
        "/tmp/kfnotepad/deep/example.md"
    );
    assert_eq!(gui_sidebar_path_label(&deep_path), ".../docs");
    assert!(gui_sidebar_path_label(&deep_path).len() <= GUI_PANEL_PATH_MAX_CHARS);
}

#[test]
fn gui_tile_window_chrome_uses_compact_gapped_layout() {
    let palette = gui_theme_palette(EditorThemeId::Terminal);
    let active_body = gui_tile_body_style(palette, true);
    let inactive_body = gui_tile_body_style(palette, false);
    let active_title = gui_tile_title_style(palette, true);
    let grid = gui_pane_grid_style(palette);

    assert_eq!(GUI_PANE_GRID_SPACING, 5.0);
    assert_eq!(GUI_EDITOR_PADDING, 2);
    assert_eq!(GUI_TILE_BODY_PADDING, 2);
    assert_eq!(GUI_TILE_TITLE_PADDING, 3);
    assert_eq!(GUI_TILE_CONTROL_SPACING, 1);
    assert_eq!(GUI_PANEL_CONTROL_SPACING, 5);
    assert_eq!(GUI_PANEL_SECTION_SPACING, 6);
    assert_eq!(GUI_PANEL_TREE_TOP_PADDING, 4.0);
    assert_eq!(GUI_CHROME_PADDING, [3, 5]);
    assert_eq!(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 32);
    assert_eq!(GUI_LINE_NUMBER_SEPARATOR_WIDTH, 1.0);
    assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
    assert_eq!(active_body.border.radius.top_left, GUI_TILE_RADIUS);
    assert_eq!(active_body.border.width, 1.0);
    assert_eq!(inactive_body.border.width, 1.0);
    assert_eq!(active_body.border.color, palette.primary);
    assert_eq!(
        inactive_body.border.color,
        Color {
            a: 0.55,
            ..palette.primary
        }
    );
    assert_eq!(active_title.text_color, Some(palette.primary));
    assert_eq!(grid.hovered_region.border.width, 0.0);
    assert_eq!(grid.hovered_region.border.color, Color::TRANSPARENT);
    assert!(matches!(
        grid.hovered_region.background,
        Background::Color(Color::TRANSPARENT)
    ));
    assert_eq!(grid.hovered_split.width, 1.0);
    assert_eq!(grid.picked_split.color, palette.primary);
}

#[test]
fn gui_editor_scrollbar_model_is_thin_and_proportional() {
    let model = gui_editor_scrollbar_model(100, 41, 20, 200.0);

    assert!(model.visible);
    assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
    assert_eq!(model.track_height, 200.0);
    assert_eq!(model.thumb_height, 40.0);
    assert_eq!(model.thumb_top, 80.0);
    assert_eq!(model.page_delta, 20);

    let hidden = gui_editor_scrollbar_model(5, 1, 20, 200.0);
    assert!(!hidden.visible);
    assert_eq!(hidden.thumb_height, 200.0);
}

#[test]
fn gui_equalized_tile_layout_places_remainder_on_right() {
    let three = equalized_tile_layout_node(3).expect("three layout");
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = three
    else {
        panic!("expected three-tile split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 500);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 2 });
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = *first
    else {
        panic!("expected first column row split");
    };
    assert_eq!(axis, GuiLayoutAxis::Horizontal);
    assert_eq!(ratio_per_mille, 500);
    assert_eq!(*first, GuiLayoutNode::Leaf { ordinal: 0 });
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 1 });

    let five = equalized_tile_layout_node(5).expect("five layout");
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = five
    else {
        panic!("expected five-tile split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 666);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
    assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
}

#[test]
fn gui_menu_tile_equalizes_visible_panes_into_grid() {
    let temp = TempArea::new("gui-equalize-menu");
    let paths = (1..=5)
        .map(|index| {
            let path = temp.path(&format!("{index}.txt"));
            fs::write(&path, format!("{index}\n")).expect("write tile");
            path
        })
        .collect::<Vec<_>>();
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: paths.clone(),
    });
    let active_path = state.workspace.active_tile().document.path.clone();

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
    );

    assert_eq!(state.status_message, "equalized 5 tiles");
    assert_eq!(state.workspace.active_tile().document.path, active_path);
    let Some(layout) = gui_layout_from_state(
        &state.panes,
        &state.workspace,
        state.browser_visible,
        state.browser_width,
    ) else {
        panic!("expected layout");
    };
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = layout.root
    else {
        panic!("expected equalized root split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 666);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
    assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
}

#[test]
fn gui_equalize_preserves_wrapped_editor_viewport_and_gutter() {
    let temp = TempArea::new("gui-equalize-renderer-viewport");
    let paths = (1..=5)
            .map(|index| {
                let path = temp.path(&format!("{index}.txt"));
                let text = (1..=120)
                    .map(|line| {
                        format!(
                            "line {line} keeps enough words around to wrap in a narrow pane without changing the source line"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(&path, text).expect("write tile");
                path
            })
            .collect::<Vec<_>>();
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: paths,
    });
    state.settings.show_line_numbers = true;
    state.settings.wrap_lines = true;

    let _ = update(&mut state, Message::ScrollActiveEditorViewport(40));
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
    );

    let surface = gui_editor_surface_model(
        state.settings,
        &state.workspace.active_tile().document,
        state.active_editor(),
        &state.syntax_highlighter,
        state.syntax_caches.get(&state.workspace.active_tile().id),
    );
    let line_numbers = surface.line_numbers.expect("visible line numbers");
    let visual_rows = gui_editor_read_only_visual_rows(
        &surface.viewport_slice.lines,
        surface.viewport_slice.first_line,
        surface.wrapping,
        34,
    );

    assert_eq!(surface.viewport_slice.first_line, 41);
    assert_eq!(line_numbers.gutter_start, 41);
    assert!(line_numbers.text.starts_with("41\n42\n43"));
    assert_eq!(visual_rows.first().map(|row| row.line.number), Some(41));
    assert!(visual_rows.iter().any(|row| !row.show_line_number));
    assert!(visual_rows.iter().all(|row| row.line.number >= 41));
    assert_eq!(state.status_message, "equalized 5 tiles");
}

#[test]
fn gui_open_replaces_initial_blank_tile_but_not_dirty_or_real_tiles() {
    let temp = TempArea::new("gui-open-replace-blank");
    let file = temp.path("opened.txt");
    fs::write(&file, "opened\n").expect("write opened file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::OpenDialogSelected(Some(file.clone())));

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
    assert_eq!(state.active_editor().text(), "opened\n");
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );

    let second = temp.path("second.txt");
    fs::write(&second, "second\n").expect("write second file");
    let _ = update(
        &mut state,
        Message::OpenDialogSelected(Some(second.clone())),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);
}

#[test]
fn gui_open_focuses_existing_file_instead_of_duplicate_tile() {
    let temp = TempArea::new("gui-open-existing-file");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first file");
    fs::write(&second, "second\n").expect("write second file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);

    let _ = update(&mut state, Message::OpenDialogSelected(Some(first.clone())));

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert_eq!(state.active_editor().text(), "first\n");
    assert!(state.status_message.starts_with("already open: "));
}

#[test]
fn gui_tile_title_controls_are_attached_for_hover_reveal_on_every_pane() {
    assert!(gui_tile_title_controls_attached(true));
    assert!(gui_tile_title_controls_attached(false));
}

#[test]
fn gui_header_layout_splits_global_actions_on_narrow_windows() {
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH),
        GuiHeaderLayoutMode::SingleRow
    );
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH + 1.0),
        GuiHeaderLayoutMode::SingleRow
    );
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH - 1.0),
        GuiHeaderLayoutMode::SplitActions
    );
}

#[test]
fn gui_search_layout_splits_controls_on_narrow_windows() {
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH),
        GuiSearchLayoutMode::SingleRow
    );
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH + 1.0),
        GuiSearchLayoutMode::SingleRow
    );
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH - 1.0),
        GuiSearchLayoutMode::SplitRows
    );
}
