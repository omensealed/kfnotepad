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
    assert_eq!(state.active_document_text(), "");
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
    assert_eq!(state.active_document_text(), "alpha\n");
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
