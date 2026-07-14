use super::*;

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
    assert_eq!(state.active_document_text(), "opened\n");
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
    assert_eq!(state.active_document_text(), "initial\n");
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
    state.replace_active_document_text("saved through dialog\n");

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
