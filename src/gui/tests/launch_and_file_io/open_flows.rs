use super::*;

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
