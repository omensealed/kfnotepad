use super::*;

#[test]
fn gui_external_file_change_refreshes_clean_tile_and_locks_editing() {
    let temp = TempArea::new("gui-external-refresh");
    let file = temp.path("watched.txt");
    fs::write(&file, "one\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file.clone()],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    let pane = state.active_pane;

    fs::write(&file, "one\ntwo\n").expect("external write");
    state.poll_external_file_changes();

    assert_eq!(state.active_editor().text(), "one\ntwo\n");
    assert!(!state.workspace.active_tile().document.buffer.is_dirty());
    assert!(state.is_external_edit_locked(tile_id));
    assert_eq!(state.status_message, "external update loaded: watched.txt");

    let _ = update(
        &mut state,
        Message::Edit(
            pane,
            text_editor::Action::Edit(text_editor::Edit::Insert('X')),
        ),
    );

    assert_eq!(state.active_editor().text(), "one\ntwo\n");
    assert_eq!(
        state.status_message,
        "external edit lock active; unlock to edit"
    );
}

#[test]
fn gui_external_file_change_detects_same_size_coarse_mtime_rewrite() {
    let temp = TempArea::new("gui-external-same-size");
    let file = temp.path("watched.txt");
    fs::write(&file, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file.clone()],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    let previous_snapshot = state
        .file_snapshots
        .get(&tile_id)
        .cloned()
        .expect("initial snapshot");

    fs::write(&file, "bravo\n").expect("same size external write");
    let current_snapshot = gui_file_snapshot(&file)
        .expect("read current snapshot")
        .expect("current file snapshot");
    assert_eq!(current_snapshot.bytes, previous_snapshot.bytes);
    assert_ne!(current_snapshot.fingerprint, previous_snapshot.fingerprint);

    state.file_snapshots.insert(
        tile_id,
        FileSnapshot {
            bytes: current_snapshot.bytes,
            modified: current_snapshot.modified,
            fingerprint: previous_snapshot.fingerprint,
        },
    );
    state.poll_external_file_changes();

    assert_eq!(state.active_editor().text(), "bravo\n");
    assert!(state.is_external_edit_locked(tile_id));
}

#[test]
fn gui_external_file_check_does_not_overlap_in_flight_scan() {
    let temp = TempArea::new("gui-external-in-flight");
    let file = temp.path("watched.txt");
    fs::write(&file, "one\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    assert!(!state.external_file_check_in_flight);
    let _first = state.request_external_file_check();
    assert!(state.external_file_check_in_flight);
    let _second = state.request_external_file_check();
    assert!(state.external_file_check_in_flight);

    state.complete_external_file_check(Vec::new());
    assert!(!state.external_file_check_in_flight);
}

#[test]
fn gui_external_file_check_uses_metadata_before_deep_snapshot() {
    let metadata = FileMetadataSnapshot {
        bytes: 12,
        modified: Some(UNIX_EPOCH + Duration::from_secs(10)),
    };
    let snapshot = FileSnapshot {
        bytes: 12,
        modified: metadata.modified,
        fingerprint: 42,
    };

    assert!(!external_file_snapshot_requires_deep_check(
        &metadata,
        Some(&snapshot),
        false,
    ));
    assert!(external_file_snapshot_requires_deep_check(
        &metadata,
        Some(&snapshot),
        true,
    ));

    let changed_metadata = FileMetadataSnapshot {
        bytes: 13,
        modified: metadata.modified,
    };
    assert!(external_file_snapshot_requires_deep_check(
        &changed_metadata,
        Some(&snapshot),
        false,
    ));
}

#[test]
fn gui_external_file_unlock_allows_editing_again() {
    let temp = TempArea::new("gui-external-unlock");
    let file = temp.path("watched.txt");
    fs::write(&file, "one\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file.clone()],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    let pane = state.active_pane;
    fs::write(&file, "external\n").expect("external write");
    state.poll_external_file_changes();

    let _ = update(&mut state, Message::UnlockExternalEdit(tile_id));
    let _ = update(
        &mut state,
        Message::Edit(
            pane,
            text_editor::Action::Edit(text_editor::Edit::Insert('X')),
        ),
    );

    assert!(!state.is_external_edit_locked(tile_id));
    assert_eq!(state.active_editor().text(), "Xexternal\n");
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
}

#[test]
fn gui_external_file_change_continues_refreshing_while_locked() {
    let temp = TempArea::new("gui-external-refresh-locked");
    let file = temp.path("watched.txt");
    fs::write(&file, "one\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file.clone()],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    fs::write(&file, "external one\n").expect("external write one");
    state.poll_external_file_changes();

    fs::write(&file, "external one\nexternal two\n").expect("external write two");
    state.poll_external_file_changes();

    assert_eq!(state.active_editor().text(), "external one\nexternal two\n");
    assert!(state.is_external_edit_locked(tile_id));
}

#[test]
fn gui_external_file_change_does_not_overwrite_dirty_tile() {
    let temp = TempArea::new("gui-external-dirty-conflict");
    let file = temp.path("watched.txt");
    fs::write(&file, "one\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file.clone()],
        },
        temp.root.clone(),
    );
    let tile_id = state.workspace.active_tile().id;
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("local dirty\n");
    state.sync_active_editor_to_document();

    fs::write(&file, "external replacement\n").expect("external write");
    state.poll_external_file_changes();

    assert_eq!(state.active_editor().text(), "local dirty\n");
    assert!(state.workspace.active_tile().document.buffer.is_dirty());
    assert!(state.is_external_edit_locked(tile_id));
    assert!(state
        .status_message
        .contains("save or close local edits before refresh"));
}
