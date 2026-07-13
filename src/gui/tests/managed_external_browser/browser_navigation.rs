use super::*;

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
