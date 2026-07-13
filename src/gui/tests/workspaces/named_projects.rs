use super::*;

#[test]
fn gui_workspace_panel_saves_named_workspace_project() {
    let temp = TempArea::new("gui-workspace-save-named");
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

    let _ = update(
        &mut state,
        Message::WorkspaceProjectNameChanged("Client Notes".to_string()),
    );
    let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

    let named_path = projects_dir.join("client-notes.v1");
    assert!(named_path.exists());
    let project = kfnotepad::parse_gui_workspace_project(
        &fs::read_to_string(named_path).expect("read named project"),
    )
    .expect("parse named project");
    assert_eq!(project.name, "Client Notes");
    assert_eq!(project.files, vec![first, second]);
    assert_eq!(project.active_ordinal, 1);
    assert!(project.layout.is_some());
    assert!(!projects_dir.join("current-workspace.v1").exists());
    assert_eq!(state.workspace_projects.len(), 1);
    assert_eq!(state.workspace_projects[0].project.name, "Client Notes");
    assert_eq!(
        state.status_message,
        "workspace project saved: Client Notes"
    );
}

#[test]
fn gui_workspace_panel_rejects_empty_or_invalid_project_names_without_writes() {
    let temp = TempArea::new("gui-workspace-save-invalid-name");
    let file = temp.path("file.txt");
    fs::write(&file, "file\n").expect("write file");
    let projects_dir = temp.path("workspaces");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir.clone());
    state.workspace_projects.clear();

    let _ = update(
        &mut state,
        Message::WorkspaceProjectNameChanged("   ".to_string()),
    );
    let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

    assert_eq!(
        state.status_message,
        "workspace save failed: project name required"
    );
    assert!(!projects_dir.exists());

    let _ = update(
        &mut state,
        Message::WorkspaceProjectNameChanged("../bad".to_string()),
    );
    let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

    assert_eq!(
        state.status_message,
        "workspace save failed: invalid project name"
    );
    assert!(!projects_dir.exists());
    assert!(state.workspace_projects.is_empty());
}

#[test]
fn gui_workspace_panel_named_save_does_not_change_current_workspace_restore_target() {
    let temp = TempArea::new("gui-workspace-save-named-current");
    let current_file = temp.path("current.txt");
    let named_file = temp.path("named.txt");
    fs::write(&current_file, "current\n").expect("write current");
    fs::write(&named_file, "named\n").expect("write named");
    let projects_dir = temp.path("workspaces");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![current_file.clone()],
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir.clone());

    let _ = update(&mut state, Message::SaveCurrentWorkspaceProject);
    state.open_path_in_new_pane(named_file.clone());
    let _ = update(
        &mut state,
        Message::WorkspaceProjectNameChanged("Named Workspace".to_string()),
    );
    let _ = update(&mut state, Message::SaveNamedWorkspaceProject);

    let current = kfnotepad::parse_gui_workspace_project(
        &fs::read_to_string(projects_dir.join("current-workspace.v1"))
            .expect("read current project"),
    )
    .expect("parse current project");
    let named = kfnotepad::parse_gui_workspace_project(
        &fs::read_to_string(projects_dir.join("named-workspace.v1")).expect("read named project"),
    )
    .expect("parse named project");

    assert_eq!(current.name, "current workspace");
    assert_eq!(current.files, vec![current_file]);
    assert_eq!(named.name, "Named Workspace");
    assert_eq!(named.files, vec![current.files[0].clone(), named_file]);
}

#[test]
fn gui_workspace_panel_refresh_lists_projects_in_deterministic_order() {
    let temp = TempArea::new("gui-workspace-refresh");
    let projects_dir = temp.path("workspaces");
    fs::create_dir_all(&projects_dir).expect("create projects dir");
    let alpha = GuiWorkspaceProject {
        name: "alpha".to_string(),
        files: vec![temp.path("alpha.md")],
        active_ordinal: 0,
        layout: None,
    };
    let zeta = GuiWorkspaceProject {
        name: "zeta".to_string(),
        files: vec![temp.path("zeta.md")],
        active_ordinal: 0,
        layout: None,
    };
    save_gui_workspace_project(&projects_dir.join("zeta.v1"), &zeta).expect("save zeta");
    save_gui_workspace_project(&projects_dir.join("alpha.v1"), &alpha).expect("save alpha");
    fs::write(projects_dir.join("broken.v1"), "bad").expect("write broken");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.workspace_projects_dir = Some(projects_dir);
    state.workspace_projects.clear();

    let _ = update(&mut state, Message::RefreshWorkspaceProjects);

    assert_eq!(
        state
            .workspace_projects
            .iter()
            .map(|entry| entry.project.name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
    assert_eq!(state.status_message, "workspace projects: 2");
}
