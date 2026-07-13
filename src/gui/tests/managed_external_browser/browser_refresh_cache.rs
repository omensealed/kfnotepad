use super::*;

#[test]
fn gui_browser_refresh_picks_up_external_file_creation() {
    let temp = TempArea::new("gui-browser-refresh");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    assert!(!state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "external.txt"));

    fs::write(temp.path("external.txt"), "external\n").expect("write external file");

    let _ = update(&mut state, Message::BrowserRefreshRequested);

    assert!(state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "external.txt"));
    assert_eq!(
        state.status_message,
        format!(
            "refreshed {}",
            temp.root.canonicalize().expect("canonical root").display()
        )
    );
}

#[test]
fn gui_file_tree_view_uses_cached_rows_until_refresh() {
    let temp = TempArea::new("gui-browser-cached-view");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let external = temp.path("external.txt");
    fs::write(&external, "external\n").expect("write external file");

    let _view = gui_file_tree_view(&state.browser_tree_rows, state.settings);
    assert!(!state
        .browser_tree_rows
        .iter()
        .any(|row| row.path() == external));

    let _ = state.refresh_file_browser();
    assert!(state
        .browser_tree_rows
        .iter()
        .any(|row| row.path() == external));
}

#[test]
fn gui_file_tree_rejects_stale_background_rows() {
    let temp = TempArea::new("gui-browser-stale-rows");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let original_rows = state.browser_tree_rows.clone();
    state.browser_tree_generation = 2;
    state.browser_tree_loading = true;

    state.apply_cached_file_tree_rows(1, Ok(Vec::new()));
    assert_eq!(state.browser_tree_rows, original_rows);
    assert!(state.browser_tree_loading);

    state.apply_cached_file_tree_rows(2, Ok(Vec::new()));
    assert!(state.browser_tree_rows.is_empty());
    assert!(!state.browser_tree_loading);
}

#[test]
fn gui_browser_rejects_stale_background_root_load() {
    let temp = TempArea::new("gui-browser-stale-root");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    let original_root = state.current_browser_dir();
    let stale = GuiBrowserLoadResult {
        browser: state.browser.as_ref().expect("browser").clone(),
        rows: Vec::new(),
        selected_path: None,
        expanded_paths: HashSet::new(),
    };
    state.browser_tree_generation = 2;
    state.browser_tree_loading = true;

    state.apply_browser_load(1, Ok(stale));

    assert_eq!(state.current_browser_dir(), original_root);
    assert!(state.browser_tree_loading);
    assert!(!state.browser_tree_rows.is_empty());
}
