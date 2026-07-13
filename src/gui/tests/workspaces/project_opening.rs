use super::*;

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
