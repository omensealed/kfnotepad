use super::*;

#[test]
fn gui_browser_clicks_are_ignored_while_workspace_panel_is_active() {
    let temp = TempArea::new("gui-left-panel-ignore-browser");
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

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
    );
    state.activate_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, initial_path);
    assert_ne!(state.workspace.active_tile().document.path, file);
}

#[test]
fn gui_browser_directory_single_click_selects_without_navigation() {
    let temp = TempArea::new("gui-browser-nav");
    fs::create_dir(temp.path("subdir")).expect("create subdir");
    fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
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
        temp.path("subdir")
    );
}
#[test]
fn gui_browser_directory_double_click_navigates_without_opening_pane() {
    let temp = TempArea::new("gui-browser-nav-double");
    fs::create_dir(temp.path("subdir")).expect("create subdir");
    fs::write(temp.path("subdir").join("inside.txt"), "inside\n").expect("write inside");
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

    state.activate_browser_entry(index);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(
        state.browser.as_ref().expect("browser").sidebar.current_dir,
        temp.path("subdir")
            .canonicalize()
            .expect("canonical subdir")
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
fn gui_browser_toggle_hides_and_restores_open_behavior() {
    let temp = TempArea::new("gui-browser-toggle");
    let file = temp.path("visible-again.txt");
    fs::write(&file, "visible\n").expect("write file");
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
        .position(|entry| entry.label == "visible-again.txt")
        .expect("browser file entry");

    let _ = update(&mut state, Message::ToggleBrowser);
    assert!(!state.browser_visible);
    state.activate_browser_entry(index);
    assert_eq!(state.workspace.tiles.len(), 1);

    let _ = update(&mut state, Message::ToggleBrowser);
    assert!(state.browser_visible);
    state.select_browser_entry(index);
    assert_ne!(state.workspace.active_tile().document.path, file);
    state.activate_browser_entry(index);
    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
}
