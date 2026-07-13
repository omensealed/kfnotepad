use super::*;

#[test]
fn gui_search_next_and_previous_update_shared_and_editor_cursor() {
    let temp = TempArea::new("gui-search-next-prev");
    let file = temp.path("search.txt");
    fs::write(&file, "alpha\nbeta alpha\n").expect("write search");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });

    let _ = update(&mut state, Message::SearchQueryChanged("alpha".to_string()));
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(&mut state, Message::SearchNext);

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
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
