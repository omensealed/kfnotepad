use super::*;

#[test]
fn gui_workspace_project_delete_requires_confirmation_and_removes_project() {
    let temp = TempArea::new("gui-workspace-delete");
    let projects_dir = temp.path("workspaces");
    let project = GuiWorkspaceProject {
        name: "delete me".to_string(),
        files: vec![temp.path("delete.md")],
        active_ordinal: 0,
        layout: None,
    };
    let project_path = projects_dir.join("delete-me.v1");
    save_gui_workspace_project(&project_path, &project).expect("save project");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir.clone());
    let _ = update(&mut state, Message::RefreshWorkspaceProjects);

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert_eq!(state.pending_project_delete, Some(0));
    assert!(project_path.exists());
    assert_eq!(
        state.status_message,
        "delete workspace project delete me? click delete again"
    );

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert_eq!(state.pending_project_delete, None);
    assert!(!project_path.exists());
    assert!(state.workspace_projects.is_empty());
    assert_eq!(
        state.status_message,
        "workspace project moved to trash: delete me"
    );
}

#[test]
fn gui_workspace_project_delete_warns_for_restore_target_when_enabled() {
    let temp = TempArea::new("gui-workspace-delete-current");
    let projects_dir = temp.path("workspaces");
    let project = GuiWorkspaceProject {
        name: "current workspace".to_string(),
        files: vec![temp.path("current.md")],
        active_ordinal: 0,
        layout: None,
    };
    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(&project_path, &project).expect("save current project");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir.clone());
    state.settings.gui_restore_last_workspace = true;
    let _ = update(&mut state, Message::RefreshWorkspaceProjects);

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert_eq!(state.pending_project_delete, Some(0));
    assert!(project_path.exists());
    assert_eq!(
        state.status_message,
        "restore target selected; delete again to remove last-workspace restore project"
    );

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert!(!project_path.exists());
    assert!(state.workspace_projects.is_empty());
    assert_eq!(
        state.status_message,
        "workspace project moved to trash: current workspace"
    );
}

#[test]
fn gui_workspace_project_delete_treats_missing_project_as_removed() {
    let temp = TempArea::new("gui-workspace-delete-missing");
    let projects_dir = temp.path("workspaces");
    fs::create_dir_all(&projects_dir).expect("create projects dir");
    let project_path = projects_dir.join("missing.v1");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir);
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: project_path,
        project: GuiWorkspaceProject {
            name: "missing".to_string(),
            files: vec![temp.path("missing.md")],
            active_ordinal: 0,
            layout: None,
        },
    }];

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));
    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert_eq!(state.pending_project_delete, None);
    assert!(state.workspace_projects.is_empty());
    assert_eq!(
        state.status_message,
        "workspace project already missing: missing"
    );
}

#[test]
fn gui_workspace_project_delete_rejects_paths_outside_project_directory() {
    let temp = TempArea::new("gui-workspace-delete-outside");
    let projects_dir = temp.path("workspaces");
    fs::create_dir_all(&projects_dir).expect("create projects dir");
    let outside_path = temp.path("outside.v1");
    fs::write(&outside_path, "outside\n").expect("write outside");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir);
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: outside_path.clone(),
        project: GuiWorkspaceProject {
            name: "outside".to_string(),
            files: vec![temp.path("outside.md")],
            active_ordinal: 0,
            layout: None,
        },
    }];

    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));
    let _ = update(&mut state, Message::WorkspaceProjectDeleteClicked(0));

    assert!(outside_path.exists());
    assert!(state
        .status_message
        .contains("workspace project path is outside the project directory"));
}
