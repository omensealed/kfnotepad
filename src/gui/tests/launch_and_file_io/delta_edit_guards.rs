use super::*;

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
fn gui_edit_overwrite_paste_uses_bulk_document_edit_and_one_mirror_rebuild() {
    let temp = TempArea::new("gui-edit-overwrite-paste-bulk");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    let active_pane = state.active_pane;
    state.replacement_overwrite_mode = true;
    state
        .panes
        .get_mut(active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 1 });

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::Edit(
            active_pane,
            text_editor::Action::Edit(text_editor::Edit::Paste("XY".to_string().into())),
        ),
    );

    assert_eq!(state.active_editor().text(), "aXYha\n");
    assert_eq!(kfnotepad::to_text_call_count(), 1);
    assert_eq!(kfnotepad::from_text_call_count(), 0);

    state.undo_active_edit();
    assert_eq!(state.active_editor().text(), "alpha\n");
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
