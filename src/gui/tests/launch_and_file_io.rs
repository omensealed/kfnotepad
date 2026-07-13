use super::*;

#[test]
fn gui_launch_loads_requested_file_into_editor_state() {
    let temp = TempArea::new("gui-open");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\nbeta\n").expect("write file");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });

    assert_eq!(state.workspace.active_tile().document.path, path);
    assert_eq!(state.active_editor().text(), "alpha\nbeta\n");
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
}

#[test]
fn gui_launches_multiple_requested_files_as_tiles_and_panes() {
    let temp = TempArea::new("gui-multi-open");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.active_editor().text(), "second\n");
    assert!(state.panes.iter().any(|(_pane, pane_state)| state
        .workspace
        .tile(pane_state.tile_id)
        .is_some_and(|tile| tile.document.path == first)));
}

#[test]
fn gui_launch_shows_startup_help_panel_without_requested_files() {
    let temp = TempArea::new("gui-startup-help");
    let state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    assert!(state.show_startup_help_panel);
}

#[test]
fn gui_launch_hides_startup_help_panel_when_files_are_requested() {
    let temp = TempArea::new("gui-startup-help-with-file");
    let first = temp.path("first.txt");
    fs::write(&first, "first\n").expect("write file");

    let state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone()],
        },
        temp.root.clone(),
    );

    assert!(!state.show_startup_help_panel);
}

#[test]
fn gui_startup_help_panel_can_be_dismissed() {
    let temp = TempArea::new("gui-startup-help-dismiss");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    assert!(state.show_startup_help_panel);
    let _ = update(&mut state, Message::DismissStartupHelp);
    assert!(!state.show_startup_help_panel);
}

#[test]
fn gui_launch_uses_balanced_halloy_style_split_axes_for_three_files() {
    let temp = TempArea::new("gui-balanced-launch");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone(), third.clone()],
    });

    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = state.panes.layout()
    else {
        panic!("expected root split");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(first));
    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = &**b
    else {
        panic!("expected active-pane nested split");
    };
    assert_eq!(*axis, pane_grid::Axis::Horizontal);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(second));
    assert_eq!(node_path(&state, b), Some(third));
}

#[test]
fn gui_file_open_splits_the_active_tall_pane_horizontally() {
    let temp = TempArea::new("gui-balanced-open");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });

    state.open_path_in_new_pane(third.clone());

    let pane_grid::Node::Split { axis, b, .. } = state.panes.layout() else {
        panic!("expected root split");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = &**b
    else {
        panic!("expected new file split inside active pane");
    };
    assert_eq!(*axis, pane_grid::Axis::Horizontal);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(second));
    assert_eq!(node_path(&state, b), Some(third.clone()));
    assert_eq!(state.workspace.active_tile().document.path, third);
    assert!(state.status_message.starts_with("opened "));
}

#[test]
fn gui_new_tile_splits_active_pane_without_creating_a_file() {
    let temp = TempArea::new("gui-new-tile");
    let existing_untitled = temp.path("untitled.txt");
    fs::write(&existing_untitled, "already here\n").expect("write existing untitled");
    let first = temp.path("first.txt");
    fs::write(&first, "first\n").expect("write first");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone()],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::NewTileRequested);

    let expected = temp.path("untitled-2.txt");
    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, expected);
    assert_eq!(state.active_editor().text(), "");
    assert!(!temp.path("untitled-2.txt").exists());
    assert_eq!(
        fs::read_to_string(&existing_untitled).expect("read existing untitled"),
        "already here\n"
    );
    assert!(state.status_message.starts_with("new tile "));
}

#[test]
fn gui_browser_width_updates_with_clamped_bounds() {
    let temp = TempArea::new("gui-browser-width");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::BrowserWidthChanged(190.0));
    assert_eq!(state.browser_width, 190.0);
    assert_eq!(state.status_message, "file browser width: 190px");

    let _ = update(&mut state, Message::BrowserWidthChanged(999.0));
    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MAX);

    let _ = update(&mut state, Message::BrowserWidthChanged(1.0));
    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MIN);
}

#[test]
fn gui_editor_sync_marks_document_dirty() {
    let temp = TempArea::new("gui-dirty");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });

    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("changed\n");
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 4 });
    state.sync_active_editor_to_document();

    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "changed\n"
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 4 }
    );
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Modified
    );
}

#[test]
fn gui_save_active_tile_uses_existing_save_adapter() {
    let temp = TempArea::new("gui-save");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved through gui\n");
    state.sync_active_editor_to_document();

    state.save_active_tile();

    assert_eq!(
        fs::read_to_string(&path).expect("read saved file"),
        "saved through gui\n"
    );
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    assert!(state.status_message.starts_with("saved "));
}

#[test]
fn gui_save_only_writes_the_focused_tile() {
    let temp = TempArea::new("gui-save-focused");
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
        .editor = GuiEditorAdapter::from_text("second changed\n");
    state.sync_active_editor_to_document();
    state.save_active_tile();

    assert_eq!(fs::read_to_string(&first).expect("read first"), "first\n");
    assert_eq!(
        fs::read_to_string(&second).expect("read second"),
        "second changed\n"
    );
}

#[test]
fn gui_edit_cursor_move_does_not_reconstruct_full_text() {
    let temp = TempArea::new("gui-edit-move-no-full-rebuild");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\nbeta\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Move(text_editor::Motion::Right),
        ),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_save_preparation_serializes_once_without_reconstructing_document() {
    let temp = TempArea::new("gui-save-single-snapshot");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("changed\n");
    state.sync_active_editor_to_document();

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _task = state.request_save_active_tile_async();

    assert_eq!(kfnotepad::to_text_call_count(), 1);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_scroll_does_not_reconstruct_full_text() {
    let temp = TempArea::new("gui-edit-scroll-no-full-rebuild");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(active_pane, text_editor::Action::Scroll { lines: 1 }),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_insert_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-insert-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Insert('X')),
        ),
    );

    assert_eq!(state.active_editor().text(), "Xalpha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_enter_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-enter-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Enter),
        ),
    );

    assert_eq!(state.active_editor().text(), "\nalpha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_backspace_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-backspace-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 2 });

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Backspace),
        ),
    );

    assert_eq!(state.active_editor().text(), "apha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_paste_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-paste-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Paste("XY".to_string().into())),
        ),
    );

    assert_eq!(state.active_editor().text(), "XYalpha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_edit_delete_uses_delta_edit_when_selection_is_empty() {
    let temp = TempArea::new("gui-edit-delete-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;

    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 2 });

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Delete),
        ),
    );

    assert_eq!(state.active_editor().text(), "alha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_replacement_inputs_do_not_reconstruct_full_text_for_typing() {
    let temp = TempArea::new("gui-replacement-input-typing-no-rebuild");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );

    assert_eq!(state.active_editor().text(), "Xalpha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_replacement_inputs_do_not_reconstruct_full_text_for_newline() {
    let temp = TempArea::new("gui-replacement-input-newline-no-rebuild");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![
            GuiEditorReplacementInput::InsertChar('X'),
            GuiEditorReplacementInput::InsertNewline,
        ]),
    );

    assert_eq!(state.active_editor().text(), "X\nalpha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
}

#[test]
fn gui_open_prompt_opens_relative_path_into_new_pane() {
    let temp = TempArea::new("gui-open-prompt");
    let initial = temp.path("initial.txt");
    let opened = temp.path("opened.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    fs::write(&opened, "opened\n").expect("write opened");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial.clone()],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenPath));
    assert_eq!(state.path_prompt, Some(GuiPathPrompt::Open));
    let _ = update(
        &mut state,
        Message::PathPromptChanged("opened.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    assert_eq!(state.path_prompt, None);
    assert_eq!(state.path_prompt_value, "");
    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, opened);
    assert_eq!(state.active_editor().text(), "opened\n");
    assert!(state.status_message.starts_with("opened "));
}

#[test]
fn gui_open_dialog_completed_opens_requested_file() {
    let temp = TempArea::new("gui-open-completed-success");
    let initial = temp.path("initial.txt");
    let opened = temp.path("opened.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    fs::write(&opened, "opened\n").expect("write opened");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial.clone()],
        },
        temp.root.clone(),
    );

    let document = open_text_file(&opened).expect("open file for completion payload");
    state.handle_open_dialog_completed(opened.clone(), Ok(document));

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, opened);
    assert_eq!(state.active_editor().text(), "opened\n");
    assert!(state.status_message.starts_with("opened "));
}

#[test]
fn gui_open_dialog_completed_error_preserves_current_tile() {
    let temp = TempArea::new("gui-open-completed-error");
    let initial = temp.path("initial.txt");
    let bad_path = temp.path("missing.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial.clone()],
        },
        temp.root.clone(),
    );

    state.handle_open_dialog_completed(
        bad_path.clone(),
        Err("simulated dialog failure".to_string()),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial);
    assert_eq!(
        state.status_message,
        format!(
            "cannot open {}: simulated dialog failure",
            bad_path.display()
        )
    );
}

#[test]
fn gui_save_as_prompt_writes_to_relative_path_and_retargets_tile() {
    let temp = TempArea::new("gui-save-as-prompt");
    let original = temp.path("original.txt");
    let target = temp.path("saved-as.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved elsewhere\n");

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAsPath));
    assert_eq!(state.path_prompt, Some(GuiPathPrompt::SaveAs));
    let _ = update(
        &mut state,
        Message::PathPromptChanged("saved-as.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    assert_eq!(state.path_prompt, None);
    assert_eq!(state.workspace.active_tile().document.path, target);
    assert_eq!(
        fs::read_to_string(temp.path("saved-as.txt")).expect("read save-as"),
        "saved elsewhere\n"
    );
    assert_eq!(
        fs::read_to_string(original).expect("read original"),
        "original\n"
    );
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    assert!(state.status_message.starts_with("saved as "));
}

#[test]
fn gui_save_as_refuses_path_already_open_in_another_tile() {
    let temp = TempArea::new("gui-save-as-duplicate");
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
        .editor = GuiEditorAdapter::from_text("second retarget attempt\n");

    let saved = state.save_active_tile_as(first.clone());

    assert!(!saved);
    assert_eq!(fs::read_to_string(&first).expect("read first"), "first\n");
    assert_eq!(
        fs::read_to_string(&second).expect("read second"),
        "second\n"
    );
    assert_eq!(
        state
            .workspace
            .tiles
            .iter()
            .filter(|tile| gui_paths_refer_to_same_file(&tile.document.path, &first))
            .count(),
        1
    );
    assert_eq!(
        state
            .workspace
            .tiles
            .iter()
            .filter(|tile| gui_paths_refer_to_same_file(&tile.document.path, &second))
            .count(),
        1
    );
    assert!(gui_paths_refer_to_same_file(
        &state.workspace.active_tile().document.path,
        &first
    ));
    assert!(state.status_message.starts_with("save as refused: "));
}

#[test]
fn gui_save_as_failure_keeps_original_tile_path_and_prompt_open() {
    let temp = TempArea::new("gui-save-as-fail");
    let original = temp.path("original.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("not saved\n");

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAsPath));
    let _ = update(
        &mut state,
        Message::PathPromptChanged("missing-parent/out.txt".to_string()),
    );
    let _ = update(&mut state, Message::SubmitPathPrompt);

    assert_eq!(state.path_prompt, Some(GuiPathPrompt::SaveAs));
    assert_eq!(
        state.workspace.active_tile().document.path,
        original.clone()
    );
    assert!(!temp.path("missing-parent").exists());
    assert_eq!(
        fs::read_to_string(original).expect("read original"),
        "original\n"
    );
    assert!(state.status_message.starts_with("save as failed: "));
}

#[test]
fn gui_save_refuses_external_modification_since_open() {
    let temp = TempArea::new("gui-save-conflict");
    let path = temp.path("note.txt");
    fs::write(&path, "original\n").expect("write original");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("gui edit\n");
    fs::write(&path, "external\n").expect("external edit");

    state.save_active_tile();

    assert_eq!(
        fs::read_to_string(&path).expect("read conflicting file"),
        "external\n"
    );
    assert!(matches!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::SaveFailed { .. }
    ));
    assert!(state
        .status_message
        .contains("file changed on disk since open or last save"));
}

#[test]
fn gui_open_request_uses_native_dialog_or_documented_fallback() {
    let temp = TempArea::new("gui-native-open-request");
    let initial = temp.path("initial.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial],
        },
        temp.root.clone(),
    );

    let unavailable_reason = KfnotepadGui::gui_file_dialog_unavailable_reason();
    let _task = update(&mut state, Message::MenuCommand(GuiMenuCommand::Open));

    match unavailable_reason {
        Some(reason) => {
            assert_eq!(state.path_prompt, Some(GuiPathPrompt::Open));
            assert_eq!(
                state.status_message,
                format!("open dialog unavailable ({reason}); using path prompt")
            );
        }
        None => {
            assert_eq!(state.path_prompt, None);
            assert_eq!(state.path_prompt_value, "");
            assert_eq!(state.status_message, "open dialog");
        }
    }
}

#[test]
fn gui_native_open_dialog_selection_uses_existing_open_adapter() {
    let temp = TempArea::new("gui-native-open-selection");
    let initial = temp.path("initial.txt");
    let opened = temp.path("opened.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    fs::write(&opened, "opened\n").expect("write opened");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial],
        },
        temp.root.clone(),
    );

    let _ = update(
        &mut state,
        Message::OpenDialogSelected(Some(opened.clone())),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, opened);
    assert_eq!(state.active_editor().text(), "opened\n");
    assert!(state.status_message.starts_with("opened "));
}

#[test]
fn gui_native_open_dialog_cancel_is_noop() {
    let temp = TempArea::new("gui-native-open-cancel");
    let initial = temp.path("initial.txt");
    fs::write(&initial, "initial\n").expect("write initial");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![initial.clone()],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::OpenDialogSelected(None));

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial);
    assert_eq!(state.active_editor().text(), "initial\n");
    assert_eq!(state.status_message, "open canceled");
}

#[test]
fn gui_save_as_request_uses_native_dialog_or_documented_fallback() {
    let temp = TempArea::new("gui-native-save-as-request");
    let original = temp.path("original.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original],
        },
        temp.root.clone(),
    );

    let unavailable_reason = KfnotepadGui::gui_file_dialog_unavailable_reason();
    let _task = update(&mut state, Message::MenuCommand(GuiMenuCommand::SaveAs));

    match unavailable_reason {
        Some(reason) => {
            assert_eq!(state.path_prompt, Some(GuiPathPrompt::SaveAs));
            assert_eq!(
                state.status_message,
                format!("save as dialog unavailable ({reason}); using path prompt")
            );
        }
        None => {
            assert_eq!(state.path_prompt, None);
            assert_eq!(state.path_prompt_value, "");
            assert_eq!(state.status_message, "save as dialog");
        }
    }
}

#[test]
fn gui_native_save_as_dialog_selection_uses_existing_save_adapter() {
    let temp = TempArea::new("gui-native-save-as-selection");
    let original = temp.path("original.txt");
    let target = temp.path("saved-as.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved through dialog\n");

    let _ = update(
        &mut state,
        Message::SaveAsDialogSelected(Some(target.clone())),
    );

    assert_eq!(state.workspace.active_tile().document.path, target);
    assert_eq!(
        fs::read_to_string(temp.path("saved-as.txt")).expect("read save-as"),
        "saved through dialog\n"
    );
    assert_eq!(
        fs::read_to_string(original).expect("read original"),
        "original\n"
    );
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    assert!(state.status_message.starts_with("saved as "));
}

#[test]
fn gui_native_save_as_dialog_cancel_is_noop() {
    let temp = TempArea::new("gui-native-save-as-cancel");
    let original = temp.path("original.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::SaveAsDialogSelected(None));

    assert_eq!(
        state.workspace.active_tile().document.path,
        original.clone()
    );
    assert_eq!(
        fs::read_to_string(original).expect("read original"),
        "original\n"
    );
    assert_eq!(state.status_message, "save as canceled");
}
