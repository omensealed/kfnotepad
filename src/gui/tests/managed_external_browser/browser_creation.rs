use super::*;

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
