use super::*;

#[test]
fn gui_managed_note_prompt_creates_and_opens_note_in_new_pane() {
    let temp = TempArea::new("gui-managed-note-prompt");
    let first = temp.path("first.txt");
    let notes_dir = temp.path("notes");
    fs::write(&first, "first\n").expect("write first");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first],
        },
        temp.root.clone(),
    );
    state.notes_dir = Some(notes_dir.clone());

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::OpenManagedNote),
    );
    assert_eq!(state.path_prompt, Some(GuiPathPrompt::ManagedNote));
    let _ = update(
        &mut state,
        Message::PathPromptChanged("Daily Note".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    let expected = notes_dir.join("daily-note.md");
    assert_eq!(state.path_prompt, None);
    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, expected);
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(
        fs::read_to_string(notes_dir.join("daily-note.md")).expect("read note"),
        ""
    );
    assert!(state.status_message.starts_with("opened note "));
}

#[test]
fn gui_managed_notes_panel_lists_and_opens_existing_note() {
    let temp = TempArea::new("gui-managed-notes-panel");
    let first = temp.path("first.txt");
    let notes_dir = temp.path("notes");
    fs::write(&first, "first\n").expect("write first");
    fs::create_dir_all(&notes_dir).expect("create notes dir");
    fs::write(notes_dir.join("alpha.md"), "alpha\n").expect("write note");
    fs::write(notes_dir.join("zeta.md"), "zeta\n").expect("write note");
    fs::write(notes_dir.join("ignore.txt"), "ignored\n").expect("write ignored");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first],
        },
        temp.root.clone(),
    );
    state.notes_dir = Some(notes_dir.clone());

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
    );

    let notes = state.notes_panel.as_ref().expect("notes panel");
    assert_eq!(
        notes
            .iter()
            .map(|note| note.file_name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha.md", "zeta.md"]
    );
    assert_eq!(state.status_message, "managed notes: 2");

    let _ = update(&mut state, Message::ManagedNoteClicked(0));

    assert_eq!(state.notes_panel, None);
    assert_eq!(
        state.workspace.active_tile().document.path,
        notes_dir.join("alpha.md")
    );
    assert_eq!(state.active_editor().text(), "alpha\n");
}

#[test]
fn gui_managed_note_delete_requires_confirmation_and_refreshes_list() {
    let temp = TempArea::new("gui-managed-note-delete");
    let first = temp.path("first.txt");
    let notes_dir = temp.path("notes");
    let alpha = notes_dir.join("alpha.md");
    fs::write(&first, "first\n").expect("write first");
    fs::create_dir_all(&notes_dir).expect("create notes dir");
    fs::write(&alpha, "alpha\n").expect("write note");
    fs::write(notes_dir.join("zeta.md"), "zeta\n").expect("write note");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first],
        },
        temp.root.clone(),
    );
    state.notes_dir = Some(notes_dir.clone());
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
    );

    let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

    assert_eq!(state.pending_managed_note_delete, Some(alpha.clone()));
    assert!(alpha.exists());
    assert_eq!(
        state.status_message,
        "delete note alpha.md? click delete again"
    );

    let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

    assert_eq!(state.pending_managed_note_delete, None);
    assert!(!alpha.exists());
    assert_eq!(
        state
            .notes_panel
            .as_ref()
            .expect("notes panel")
            .iter()
            .map(|note| note.file_name.as_str())
            .collect::<Vec<_>>(),
        vec!["zeta.md"]
    );
    assert_eq!(
        state.status_message,
        "managed note moved to trash: alpha.md"
    );
}
#[test]
fn gui_managed_note_delete_refuses_open_note_tile() {
    let temp = TempArea::new("gui-managed-note-delete-open");
    let first = temp.path("first.txt");
    let notes_dir = temp.path("notes");
    let alpha = notes_dir.join("alpha.md");
    fs::write(&first, "first\n").expect("write first");
    fs::create_dir_all(&notes_dir).expect("create notes dir");
    fs::write(&alpha, "alpha\n").expect("write note");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first],
        },
        temp.root.clone(),
    );
    state.notes_dir = Some(notes_dir.clone());
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
    );
    let _ = update(&mut state, Message::ManagedNoteClicked(0));
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ListManagedNotes),
    );

    let _ = update(&mut state, Message::ManagedNoteDeleteClicked(0));

    assert_eq!(state.pending_managed_note_delete, None);
    assert!(alpha.exists());
    assert_eq!(
        state.status_message,
        "close note tile before deleting alpha.md"
    );
}

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

#[test]
fn gui_browser_file_single_click_selects_without_opening_tile() {
    let temp = TempArea::new("gui-browser-open");
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

    state.select_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial_path);
    assert_eq!(
        state
            .browser
            .as_ref()
            .expect("browser")
            .selected_entry()
            .expect("selected")
            .path,
        file
    );
}

#[test]
fn gui_browser_file_double_click_replaces_initial_blank_tile() {
    let temp = TempArea::new("gui-browser-open-double");
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
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "from-browser.txt")
        .expect("browser file entry");

    state.activate_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
    assert_eq!(state.active_editor().text(), "browser file\n");
}

#[test]
fn gui_browser_tree_file_selection_does_not_open_tile() {
    let temp = TempArea::new("gui-browser-tree-file");
    let file = temp.path("from-tree.txt");
    fs::write(&file, "tree file\n").expect("write browser file");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeSelected(file.clone(), false),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial_path);
    assert_eq!(
        state
            .browser
            .as_ref()
            .expect("browser")
            .selected_entry()
            .expect("selected")
            .path,
        file
    );
    assert_eq!(state.browser_selected_path.as_deref(), Some(file.as_path()));
}

#[test]
fn gui_browser_tree_file_double_click_uses_existing_open_adapter() {
    let temp = TempArea::new("gui-browser-tree-file-double");
    let file = temp.path("from-tree.txt");
    fs::write(&file, "tree file\n").expect("write browser file");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeActivated(file.clone(), false),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
    assert_eq!(state.active_editor().text(), "tree file\n");
}

#[test]
fn gui_browser_tree_directory_selection_does_not_reset_root() {
    let temp = TempArea::new("gui-browser-tree-dir");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
    fs::write(subdir.join("inside.txt"), "inside\n").expect("write inside");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeSelected(subdir.clone(), true),
    );

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
        subdir
    );
    assert_eq!(
        state.browser_selected_path.as_deref(),
        Some(subdir.as_path())
    );
}

#[test]
fn gui_browser_tree_directory_double_click_resets_root_without_opening_tile() {
    let temp = TempArea::new("gui-browser-tree-dir-double");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
    fs::write(subdir.join("inside.txt"), "inside\n").expect("write inside");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeActivated(subdir.clone(), true),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(
        state.browser.as_ref().expect("browser").sidebar.current_dir,
        subdir.canonicalize().expect("canonical subdir")
    );
    assert_eq!(
        state
            .browser_tree_rows
            .first()
            .expect("tree root")
            .path
            .clone(),
        subdir.canonicalize().expect("canonical tree subdir")
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
fn gui_browser_parent_request_resets_tree_root_to_parent_directory() {
    let temp = TempArea::new("gui-browser-tree-parent");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        subdir.clone(),
    );

    let _ = update(&mut state, Message::BrowserParentRequested);

    let parent = temp.root.canonicalize().expect("canonical parent");
    assert_eq!(
        state.browser.as_ref().expect("browser").sidebar.current_dir,
        parent
    );
    assert_eq!(
        state
            .browser_tree_rows
            .first()
            .expect("tree root")
            .path
            .clone(),
        parent
    );
}

#[test]
fn gui_browser_refresh_picks_up_external_file_creation() {
    let temp = TempArea::new("gui-browser-refresh");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    assert!(!state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "external.txt"));

    fs::write(temp.path("external.txt"), "external\n").expect("write external file");

    let _ = update(&mut state, Message::BrowserRefreshRequested);

    assert!(state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "external.txt"));
    assert_eq!(
        state.status_message,
        format!(
            "refreshed {}",
            temp.root.canonicalize().expect("canonical root").display()
        )
    );
}

#[test]
fn gui_file_tree_view_uses_cached_rows_until_refresh() {
    let temp = TempArea::new("gui-browser-cached-view");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let external = temp.path("external.txt");
    fs::write(&external, "external\n").expect("write external file");

    let _view = gui_file_tree_view(&state.browser_tree_rows, state.settings);
    assert!(!state
        .browser_tree_rows
        .iter()
        .any(|row| row.path() == external));

    let _ = state.refresh_file_browser();
    assert!(state
        .browser_tree_rows
        .iter()
        .any(|row| row.path() == external));
}

#[test]
fn gui_file_tree_rejects_stale_background_rows() {
    let temp = TempArea::new("gui-browser-stale-rows");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let original_rows = state.browser_tree_rows.clone();
    state.browser_tree_generation = 2;
    state.browser_tree_loading = true;

    state.apply_cached_file_tree_rows(1, Ok(Vec::new()));
    assert_eq!(state.browser_tree_rows, original_rows);
    assert!(state.browser_tree_loading);

    state.apply_cached_file_tree_rows(2, Ok(Vec::new()));
    assert!(state.browser_tree_rows.is_empty());
    assert!(!state.browser_tree_loading);
}

#[test]
fn gui_browser_rejects_stale_background_root_load() {
    let temp = TempArea::new("gui-browser-stale-root");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let original_root = state.current_browser_dir();
    let stale = GuiBrowserLoadResult {
        browser: state.browser.as_ref().expect("browser").clone(),
        rows: Vec::new(),
        selected_path: None,
        expanded_paths: HashSet::new(),
    };
    state.browser_tree_generation = 2;
    state.browser_tree_loading = true;

    state.apply_browser_load(1, Ok(stale));

    assert_eq!(state.current_browser_dir(), original_root);
    assert!(state.browser_tree_loading);
    assert!(!state.browser_tree_rows.is_empty());
}

#[test]
fn gui_browser_create_file_creates_refreshes_and_opens_new_file() {
    let temp = TempArea::new("gui-browser-create-file");
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
    let created = temp.path("created.txt");

    let _ = update(&mut state, Message::BrowserCreateFileRequested);
    assert_eq!(state.path_prompt, Some(GuiPathPrompt::BrowserCreateFile));
    let _ = update(
        &mut state,
        Message::PathPromptChanged("created.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    assert!(created.exists());
    assert_eq!(fs::read_to_string(&created).expect("read created file"), "");
    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, created);
    assert_eq!(state.active_editor().text(), "");
    assert!(state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "created.txt"));
    assert_eq!(
        state.status_message,
        format!("created {}", created.display())
    );
}

#[test]
fn gui_browser_create_file_targets_selected_directory() {
    let temp = TempArea::new("gui-browser-create-file-selected-dir");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
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

    let _ = update(&mut state, Message::BrowserCreateFileRequested);
    let _ = update(
        &mut state,
        Message::PathPromptChanged("nested.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    let created = subdir.join("nested.txt");
    assert!(created.exists());
    assert_eq!(state.workspace.active_tile().document.path, created);
    assert_eq!(
        state.status_message,
        format!("created {}", created.display())
    );
}

#[test]
fn gui_browser_create_directory_targets_selected_directory() {
    let temp = TempArea::new("gui-browser-create-dir-selected-dir");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
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

    let _ = update(&mut state, Message::BrowserCreateDirectoryRequested);
    assert_eq!(
        state.path_prompt,
        Some(GuiPathPrompt::BrowserCreateDirectory)
    );
    let _ = update(&mut state, Message::PathPromptChanged("child".to_string()));
    let _ = update(&mut state, Message::SubmitPathPrompt);

    let created = subdir.join("child");
    assert!(created.is_dir());
    assert_eq!(
        state.status_message,
        format!("created directory {}", created.display())
    );
}

#[test]
fn gui_browser_create_file_targets_tree_selected_nested_directory() {
    let temp = TempArea::new("gui-browser-create-file-tree-selected-dir");
    let subdir = temp.path("subdir");
    let nested = subdir.join("nested");
    fs::create_dir(&subdir).expect("create subdir");
    fs::create_dir(&nested).expect("create nested");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeSelected(nested.clone(), true),
    );
    let _ = update(&mut state, Message::BrowserCreateFileRequested);
    let _ = update(
        &mut state,
        Message::PathPromptChanged("created.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    let created = nested.join("created.txt");
    assert!(created.exists());
    assert_eq!(state.workspace.active_tile().document.path, created);
    assert_eq!(
        state.browser_selected_path.as_deref(),
        Some(created.as_path())
    );
}

#[test]
fn gui_browser_delete_file_requires_confirmation() {
    let temp = TempArea::new("gui-browser-delete-file");
    let file = temp.path("delete-me.txt");
    fs::write(&file, "delete\n").expect("write delete file");
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
        .position(|entry| entry.label == "delete-me.txt")
        .expect("delete file entry");
    state.select_browser_entry(index);

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(file.exists());
    assert!(state.status_message.contains("click delete again"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!file.exists());
    assert_eq!(
        state.status_message,
        format!("moved file to trash {}", file.display())
    );
}

#[test]
fn gui_browser_delete_targets_tree_selected_nested_file() {
    let temp = TempArea::new("gui-browser-delete-tree-file");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
    let file = subdir.join("delete-me.txt");
    fs::write(&file, "delete\n").expect("write delete file");
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

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeSelected(file.clone(), false),
    );
    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(file.exists());
    assert!(state.status_message.contains("click delete again"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!file.exists());
    assert_eq!(
        state.status_message,
        format!("moved file to trash {}", file.display())
    );
}

#[test]
fn gui_browser_delete_directory_warns_and_removes_tree_after_confirmation() {
    let temp = TempArea::new("gui-browser-delete-dir");
    let directory = temp.path("delete-dir");
    fs::create_dir(&directory).expect("create delete dir");
    fs::create_dir(directory.join("child")).expect("create child dir");
    fs::write(directory.join("child").join("nested.txt"), "nested\n").expect("write nested");
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
        .position(|entry| entry.label == "delete-dir/")
        .expect("delete dir entry");
    state.select_browser_entry(index);

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(directory.exists());
    assert!(state.status_message.contains("all subdirectories/files"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!directory.exists());
    assert_eq!(
        state.status_message,
        format!("moved directory to trash {}", directory.display())
    );
}
