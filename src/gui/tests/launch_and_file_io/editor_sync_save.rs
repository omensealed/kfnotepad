use super::*;

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
