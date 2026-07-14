use super::*;

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
    state.replace_active_document_text("saved elsewhere\n");

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
    state.replace_active_document_text("second retarget attempt\n");

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
    state.replace_active_document_text("not saved\n");

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
    state.replace_active_document_text("gui edit\n");
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
