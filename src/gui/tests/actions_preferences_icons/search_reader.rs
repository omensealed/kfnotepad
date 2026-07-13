use super::*;

#[test]
fn gui_search_defaults_case_insensitive_and_can_toggle_sensitive() {
    let temp = TempArea::new("gui-search-case");
    let config = temp.path("config.toml");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "Heading\nAlpha only\n").expect("write note");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![file_path],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        None,
    );
    state.search_query = "alpha".to_string();

    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "found next: alpha");
    assert_eq!(state.workspace.active_tile().state.cursor.row, 1);
    assert_eq!(state.workspace.active_tile().state.cursor.column, 0);

    let _ = update(&mut state, Message::SearchCaseSensitiveChanged(true));
    assert!(state.settings.search_case_sensitive);
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "no match: alpha");
    assert!(fs::read_to_string(&config)
        .expect("read config")
        .contains("search_case_sensitive = true\n"));
}

#[test]
fn gui_case_insensitive_search_maps_expanded_unicode_to_original_columns() {
    let temp = TempArea::new("gui-search-unicode-case");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "aßb SS\nİstanbul\n").expect("write note");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file_path],
    });

    let _ = update(&mut state, Message::SearchQueryChanged("ss".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 1 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("ß"));

    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 4 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("SS"));

    let _ = update(&mut state, Message::SearchQueryChanged("i".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("İ"));
}

#[test]
fn gui_case_sensitive_search_selects_full_grapheme_for_partial_match() {
    let temp = TempArea::new("gui-search-grapheme-case");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "🇺🇸 e\u{301}x\n").expect("write note");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file_path],
    });

    let _ = update(&mut state, Message::SearchCaseSensitiveChanged(true));
    let _ = update(&mut state, Message::SearchQueryChanged("🇸".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("🇺🇸"));

    let _ = update(
        &mut state,
        Message::SearchQueryChanged("\u{301}".to_string()),
    );
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 3 }
    );
    assert_eq!(
        state.active_editor().selection().as_deref(),
        Some("e\u{301}")
    );
}

#[test]
fn gui_reader_mode_ticks_scroll_active_document_and_persists_speed() {
    let temp = TempArea::new("gui-reader-mode");
    let config = temp.path("config.toml");
    let file_path = temp.path("long.txt");
    fs::write(
        &file_path,
        (1..=80)
            .map(|line| format!("line {line}\n"))
            .collect::<String>(),
    )
    .expect("write long note");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![file_path],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        None,
    );

    let _ = update(&mut state, Message::ReaderSpeedChanged(120));
    let _ = update(&mut state, Message::ReaderModeChanged(true));
    let before = state
        .panes
        .get(state.active_pane)
        .expect("active pane")
        .editor
        .viewport
        .first_line;
    let _ = update(&mut state, Message::ReaderScrollTick);
    let after = state
        .panes
        .get(state.active_pane)
        .expect("active pane")
        .editor
        .viewport
        .first_line;

    assert!(after > before);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .expect("active pane")
            .editor
            .document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert!(state.settings.gui_reader_mode_enabled);
    assert_eq!(state.settings.gui_reader_lines_per_minute, 120);
    let saved = fs::read_to_string(&config).expect("read config");
    assert!(saved.contains("gui_reader_mode_enabled = true\n"));
    assert!(saved.contains("gui_reader_lines_per_minute = 120\n"));
}
