use super::*;

#[test]
fn gui_left_panel_switches_between_files_workspaces_and_preferences_without_project_io() {
    let temp = TempArea::new("gui-left-panel-mode");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    assert!(state.left_panel.visible);
    assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Files);
    assert!(state.browser_visible);
    let initial_tiles = state.workspace.tiles.len();

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
    );

    assert!(state.left_panel.visible);
    assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Workspaces);
    assert!(state.browser_visible);
    assert_eq!(state.workspace.tiles.len(), initial_tiles);
    assert_eq!(state.status_message, "workspaces panel shown");

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Preferences),
    );
    assert!(state.left_panel.visible);
    assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Preferences);
    assert!(state.browser_visible);
    assert_eq!(state.workspace.tiles.len(), initial_tiles);
    assert_eq!(state.status_message, "preferences panel shown");

    let _ = update(&mut state, Message::ToggleBrowser);
    assert!(!state.left_panel.visible);
    assert!(!state.browser_visible);
    assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Preferences);

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Files),
    );
    assert!(state.left_panel.visible);
    assert_eq!(state.left_panel.mode, GuiLeftPanelMode::Files);
    assert!(state.browser_visible);
    assert_eq!(state.workspace.tiles.len(), initial_tiles);
}

#[test]
fn gui_workspace_panel_saves_current_workspace_project() {
    let temp = TempArea::new("gui-workspace-save-current");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let projects_dir = temp.path("workspaces");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir.clone());
    state.workspace_projects.clear();

    let _ = update(
        &mut state,
        Message::SelectLeftPanelMode(GuiLeftPanelMode::Workspaces),
    );
    let _ = update(&mut state, Message::SaveCurrentWorkspaceProject);

    let project_path = projects_dir.join("current-workspace.v1");
    let text = fs::read_to_string(project_path).expect("read saved project");
    let project = kfnotepad::parse_gui_workspace_project(&text).expect("parse project");
    assert_eq!(project.name, "current workspace");
    assert_eq!(project.files, vec![first, second]);
    assert_eq!(project.active_ordinal, 1);
    assert!(project.layout.is_some());
    assert_eq!(state.workspace_projects.len(), 1);
    assert_eq!(
        state.workspace_projects[0].project.name,
        "current workspace"
    );
    assert_eq!(
        state.status_message,
        "workspace project saved: current workspace"
    );
}
