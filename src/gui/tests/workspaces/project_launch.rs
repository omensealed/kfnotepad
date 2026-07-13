use super::*;

#[test]
fn gui_workspace_project_new_window_spawns_current_executable_with_project_env() {
    let temp = TempArea::new("gui-workspace-new-window");
    let project_path = temp.path("workspaces").join("project.v1");
    let file = temp.path("project.txt");
    fs::write(&file, "project\n").expect("write project");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: project_path.clone(),
        project: GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![file],
            active_ordinal: 0,
            layout: None,
        },
    }];

    let _ = update(&mut state, Message::WorkspaceProjectNewWindowClicked(0));

    assert_eq!(state.spawned_workspace_project_paths, vec![project_path]);
    assert_eq!(
        state.status_message,
        "opened workspace project project in new window"
    );
}

#[test]
fn gui_workspace_project_launch_command_uses_env_without_shell_arguments() {
    let executable = PathBuf::from("/tmp/kfnotepad-gui");
    let project = PathBuf::from("/tmp/project with spaces.v1");
    let command = workspace_project_launch_command(&executable, &project);

    assert_eq!(command.get_program(), executable.as_os_str());
    assert!(command.get_args().next().is_none());
    assert_eq!(
        command
            .get_envs()
            .find(|(name, _value)| *name == WORKSPACE_PROJECT_ENV)
            .and_then(|(_name, value)| value),
        Some(project.as_os_str())
    );
}

#[test]
fn gui_workspace_project_launch_restores_saved_files_layout_and_active_tile() {
    let temp = TempArea::new("gui-workspace-project-launch");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let project_path = temp.path("workspaces").join("project.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let project = GuiWorkspaceProject {
        name: "project".to_string(),
        files: vec![first.clone(), second.clone()],
        active_ordinal: 0,
        layout: Some(GuiLayout {
            browser_visible: false,
            browser_width_px: Some(260),
            root: GuiLayoutNode::Split {
                axis: GuiLayoutAxis::Horizontal,
                ratio_per_mille: 300,
                first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
            },
            minimized_ordinals: vec![1],
        }),
    };
    save_gui_workspace_project(&project_path, &project).expect("save project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        Some(project_path),
        None,
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert!(!state.browser_visible);
    assert_eq!(state.browser_width, 260.0);
    assert!(
        state
            .workspace
            .tiles
            .iter()
            .find(|tile| tile.document.path == second)
            .expect("second tile")
            .minimized
    );
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.minimized_panes.len(), 1);
    assert_eq!(state.minimized_panes[0].tile_id, GuiTileId(1));
    assert!(matches!(state.panes.layout(), pane_grid::Node::Pane(_)));
    assert!(state
        .status_message
        .contains("opened workspace project project"));
}
