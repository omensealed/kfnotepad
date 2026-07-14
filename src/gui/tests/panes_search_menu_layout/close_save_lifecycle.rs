use super::*;

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
    state.replace_active_document_text("discard me\n");

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
    state.replace_active_document_text("discard from app close\n");

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
    state.replace_active_document_text("dirty from ctrl-q\n");

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
    state.replace_active_document_text("saved before close\n");

    let _task = update(
        &mut state,
        Message::WindowCloseRequested(window::Id::unique()),
    );
    assert!(state.pending_app_quit);
    let tile_id = state.workspace.active_tile().id;
    let _ = update(&mut state, Message::SaveRequested);
    let source_revision = state
        .workspace
        .active_tile()
        .document
        .buffer
        .edit_revision();
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
    state.replace_active_document_text("queued save text\n");

    let _task = update(&mut state, Message::SaveRequested);
    let source_revision = state
        .workspace
        .active_tile()
        .document
        .buffer
        .edit_revision();
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
    state.replace_active_document_text("saved\n");
    let tile_id = state.workspace.active_tile().id;

    let _first = update(&mut state, Message::SaveRequested);
    let source_revision = state
        .workspace
        .active_tile()
        .document
        .buffer
        .edit_revision();
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

    let follow_up_revision = state
        .workspace
        .active_tile()
        .document
        .buffer
        .edit_revision();
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
