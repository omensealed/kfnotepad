use super::*;

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
