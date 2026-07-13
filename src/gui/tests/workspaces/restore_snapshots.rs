use super::*;

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
