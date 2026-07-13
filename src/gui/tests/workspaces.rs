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

#[test]
fn gui_restore_last_workspace_toggle_saves_current_workspace_for_relaunch() {
    let temp = TempArea::new("gui-restore-toggle-autosave");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        Some(projects_dir.clone()),
    );

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));

    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    assert!(project_path.exists());
    let restored = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(restored.workspace.tiles.len(), 2);
    assert_eq!(restored.workspace.active_tile().document.path, second);
    assert_eq!(restored.active_editor().text(), "second\n");
    assert!(restored
        .status_message
        .contains("restored last workspace project current workspace"));
}

#[test]
fn gui_restore_last_workspace_updates_snapshot_after_later_file_open() {
    let temp = TempArea::new("gui-restore-open-autosave");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let opened = temp.path("opened.txt");
    fs::write(&opened, "opened later\n").expect("write opened");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        Some(projects_dir.clone()),
    );

    let _ = update(&mut state, Message::RestoreLastWorkspaceChanged(true));
    assert_eq!(state.active_editor().text(), "");

    assert!(state.open_path_in_new_pane(opened.clone()));

    let restored = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(restored.workspace.tiles.len(), 1);
    assert_eq!(restored.workspace.active_tile().document.path, opened);
    assert_eq!(restored.active_editor().text(), "opened later\n");
    assert!(restored
        .status_message
        .contains("restored last workspace project current workspace"));
}

#[test]
fn gui_restore_last_workspace_updates_snapshot_from_explicit_launch_files() {
    let temp = TempArea::new("gui-restore-explicit-launch-autosave");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first launch\n").expect("write first");
    fs::write(&second, "second launch\n").expect("write second");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");

    let launched = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        Some(projects_dir.clone()),
    );
    assert_eq!(launched.workspace.tiles.len(), 2);
    assert_eq!(launched.workspace.active_tile().document.path, second);
    assert!(!launched.status_message.contains("restored last workspace"));

    let restored = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(restored.workspace.tiles.len(), 2);
    assert_eq!(restored.workspace.active_tile().document.path, second);
    assert_eq!(restored.active_editor().text(), "second launch\n");
    assert!(restored
        .status_message
        .contains("restored last workspace project current workspace"));
}

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

#[test]
fn gui_workspace_project_click_opens_clean_project_in_current_window() {
    let temp = TempArea::new("gui-workspace-open-clean");
    let original = temp.path("original.txt");
    let first = temp.path("project-first.txt");
    let second = temp.path("project-second.txt");
    fs::write(&original, "original\n").expect("write original");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original],
        },
        temp.root.clone(),
    );
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: temp.path("workspaces").join("project.v1"),
        project: GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![first.clone(), second.clone()],
            active_ordinal: 1,
            layout: None,
        },
    }];

    let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(
        state
            .workspace
            .tiles
            .iter()
            .map(|tile| tile.document.path.clone())
            .collect::<Vec<_>>(),
        vec![first, second.clone()]
    );
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.active_editor().text(), "second\n");
    assert_eq!(state.status_message, "opened workspace project project");
}

#[test]
fn gui_workspace_project_click_requires_confirmation_for_dirty_workspace() {
    let temp = TempArea::new("gui-workspace-open-dirty");
    let original = temp.path("original.txt");
    let project_file = temp.path("project.txt");
    fs::write(&original, "original\n").expect("write original");
    fs::write(&project_file, "project\n").expect("write project");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );
    state.panes.get_mut(state.active_pane).expect("pane").editor =
        GuiEditorAdapter::from_text("dirty\n");
    state.sync_active_editor_to_document();
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: temp.path("workspaces").join("project.v1"),
        project: GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![project_file.clone()],
            active_ordinal: 0,
            layout: None,
        },
    }];

    let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

    assert_eq!(state.workspace.active_tile().document.path, original);
    assert_eq!(state.pending_project_open, Some(0));
    assert_eq!(
        state.status_message,
        "unsaved changes; open project again to replace current workspace"
    );

    let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

    assert_eq!(state.pending_project_open, None);
    assert_eq!(state.workspace.active_tile().document.path, project_file);
    assert_eq!(state.active_editor().text(), "project\n");
}

#[test]
fn gui_workspace_project_click_skips_missing_files_and_loads_available_tiles() {
    let temp = TempArea::new("gui-workspace-open-partial");
    let original = temp.path("original.txt");
    let available = temp.path("available.txt");
    let missing = temp.path("missing.txt");
    fs::write(&original, "original\n").expect("write original");
    fs::write(&available, "available\n").expect("write available");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original.clone()],
        },
        temp.root.clone(),
    );
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: temp.path("workspaces").join("project.v1"),
        project: GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![missing.clone(), available.clone()],
            active_ordinal: 1,
            layout: None,
        },
    }];
    state.pending_project_open = Some(0);

    let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.pending_project_open, None);
    assert_eq!(state.workspace.active_tile().document.path, available);
    assert_eq!(state.active_editor().text(), "available\n");
    assert!(state
        .status_message
        .contains("opened workspace project project"));
    assert!(state
        .status_message
        .contains("skipped 1 missing/unavailable workspace file(s)"));
    assert!(state
        .status_message
        .contains(&missing.display().to_string()));
}

#[test]
fn gui_workspace_project_click_uses_blank_tile_when_no_files_load() {
    let temp = TempArea::new("gui-workspace-open-all-missing");
    let original = temp.path("original.txt");
    let missing = temp.path("missing.txt");
    fs::write(&original, "original\n").expect("write original");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![original],
        },
        temp.root.clone(),
    );
    state.workspace_projects = vec![GuiWorkspaceProjectEntry {
        path: temp.path("workspaces").join("project.v1"),
        project: GuiWorkspaceProject {
            name: "project".to_string(),
            files: vec![missing],
            active_ordinal: 0,
            layout: None,
        },
    }];
    state.pending_project_open = Some(0);

    let _ = update(&mut state, Message::WorkspaceProjectClicked(0));

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(
        state.workspace.active_tile().document.path,
        temp.root.join("untitled.txt")
    );
    assert_eq!(state.active_editor().text(), "");
    assert!(state.status_message.contains("opened blank tile"));
}

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

#[test]
fn gui_workspace_auto_restore_is_disabled_by_default() {
    let temp = TempArea::new("gui-auto-restore-disabled");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let file = temp.path("saved.txt");
    fs::write(&file, "saved\n").expect("write saved file");
    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![file.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        Some(projects_dir),
    );

    assert_ne!(state.workspace.active_tile().document.path, file);
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(state.status_message, "started empty GUI document tile");
}

#[test]
fn gui_workspace_auto_restore_opens_current_workspace_when_enabled() {
    let temp = TempArea::new("gui-auto-restore-enabled");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![first.clone(), second.clone()],
            active_ordinal: 1,
            layout: Some(GuiLayout {
                browser_visible: false,
                browser_width_px: Some(240),
                root: GuiLayoutNode::Split {
                    axis: GuiLayoutAxis::Vertical,
                    ratio_per_mille: 600,
                    first: Box::new(GuiLayoutNode::Leaf { ordinal: 0 }),
                    second: Box::new(GuiLayoutNode::Leaf { ordinal: 1 }),
                },
                minimized_ordinals: vec![0],
            }),
        },
    )
    .expect("save project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.active_editor().text(), "second\n");
    assert!(!state.browser_visible);
    assert_eq!(state.browser_width, 240.0);
    assert!(
        state
            .workspace
            .tiles
            .iter()
            .find(|tile| tile.document.path == first)
            .expect("first tile")
            .minimized
    );
    assert!(state
        .status_message
        .contains("restored last workspace project current workspace"));
}

#[test]
fn gui_workspace_auto_restore_yields_to_explicit_file_args() {
    let temp = TempArea::new("gui-auto-restore-file-precedence");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let saved = temp.path("saved.txt");
    let explicit = temp.path("explicit.txt");
    fs::write(&saved, "saved\n").expect("write saved");
    fs::write(&explicit, "explicit\n").expect("write explicit");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![saved],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![explicit.clone()],
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, explicit);
    assert_eq!(state.active_editor().text(), "explicit\n");
    assert!(!state.status_message.contains("restored last workspace"));
}

#[test]
fn gui_workspace_auto_restore_yields_to_explicit_project_launch() {
    let temp = TempArea::new("gui-auto-restore-project-precedence");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let current = temp.path("current.txt");
    let explicit = temp.path("explicit.txt");
    fs::write(&current, "current\n").expect("write current");
    fs::write(&explicit, "explicit\n").expect("write explicit");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let current_project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(
        &current_project_path,
        &GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![current],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save current project");
    let explicit_project = temp.path("explicit-project.v1");
    save_gui_workspace_project(
        &explicit_project,
        &GuiWorkspaceProject {
            name: "explicit".to_string(),
            files: vec![explicit.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save explicit project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        Some(explicit_project),
        Some(projects_dir),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, explicit);
    assert_eq!(state.active_editor().text(), "explicit\n");
    assert!(state
        .status_message
        .contains("opened workspace project explicit"));
    assert!(!state.status_message.contains("restored last workspace"));
}

#[test]
fn gui_workspace_auto_restore_invalid_path_opens_blank_without_writing_files() {
    let temp = TempArea::new("gui-auto-restore-invalid");
    let config = temp.path("config").join("kfnotepad").join("config.toml");
    let projects_dir = temp.path("config").join("kfnotepad").join("workspaces");
    let existing = temp.path("existing.txt");
    let missing = temp.path("missing.txt");
    fs::write(&existing, "unchanged\n").expect("write existing");
    fs::create_dir_all(config.parent().expect("config parent")).expect("config dir");
    fs::write(
            &config,
            "theme = \"nocturne\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = true\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\n",
        )
        .expect("write config");
    let project_path =
        gui_workspace_project_path(&projects_dir, "current workspace").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "current workspace".to_string(),
            files: vec![missing.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save project");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        Some(config),
        None,
        None,
        Some(projects_dir),
    );

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(
        state.workspace.active_tile().document.path,
        temp.root.join("untitled.txt")
    );
    assert!(state
        .status_message
        .contains("restored last workspace project current workspace"));
    assert!(state.status_message.contains("opened blank tile"));
    assert_eq!(
        fs::read_to_string(existing).expect("read existing"),
        "unchanged\n"
    );
    let repaired_project =
        load_workspace_project_launch(&project_path).expect("read repaired current project");
    assert_eq!(
        repaired_project.files,
        vec![state.workspace.active_tile().document.path.clone()]
    );
    assert_ne!(repaired_project.files, vec![missing]);
}
