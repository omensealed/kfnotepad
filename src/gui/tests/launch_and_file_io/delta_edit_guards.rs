use super::*;

#[test]
fn gui_edit_cursor_move_does_not_reconstruct_full_text() {
    let temp = TempArea::new("gui-edit-move-no-full-rebuild");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\nbeta\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::Move(
            kfnotepad::CursorMove::Right,
        )]),
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
    state.replace_active_document_text("changed\n");

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
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(&mut state, Message::ScrollActiveEditorViewport(1));

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
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "Xalpha\n");
}

#[test]
fn gui_edit_enter_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-enter-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertNewline]),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "\nalpha\n");
}

#[test]
fn gui_edit_backspace_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-backspace-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 2 });
    state.sync_pane_cursor_to_document(state.active_pane);

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::DeleteBackward]),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "apha\n");
}

#[test]
fn gui_edit_paste_uses_delta_edit_and_no_rebuild() {
    let temp = TempArea::new("gui-edit-paste-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(&mut state, Message::ClipboardPasted(Some("XY".to_string())));

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "XYalpha\n");
}

#[test]
fn gui_edit_overwrite_paste_uses_bulk_document_edit_without_full_serialization() {
    let temp = TempArea::new("gui-edit-overwrite-paste-bulk");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    state.replacement_overwrite_mode = true;
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 1 });
    state.sync_pane_cursor_to_document(state.active_pane);

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(&mut state, Message::ClipboardPasted(Some("XY".to_string())));

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "aXYha\n");

    state.undo_active_edit();
    assert_eq!(state.active_document_text(), "alpha\n");
}

#[test]
fn gui_edit_delete_uses_delta_edit_when_selection_is_empty() {
    let temp = TempArea::new("gui-edit-delete-rebuilds-text");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor
        .move_to(DocumentCursor { row: 0, column: 2 });
    state.sync_pane_cursor_to_document(state.active_pane);

    kfnotepad::reset_to_text_call_count();
    kfnotepad::reset_from_text_call_count();
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::DeleteForward]),
    );

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "alha\n");
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

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "Xalpha\n");
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

    assert_eq!(kfnotepad::to_text_call_count(), 0);
    assert_eq!(kfnotepad::from_text_call_count(), 0);
    assert_eq!(state.active_document_text(), "X\nalpha\n");
}
