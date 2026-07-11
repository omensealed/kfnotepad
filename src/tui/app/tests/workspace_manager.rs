use super::*;
use crate::tui::input::*;

#[test]
fn tui_workspace_manager_opens_for_many_projects() {
    let temp = TempArea::new("tui-workspace-manager-many");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "note\n").expect("write note");
    let projects_dir = temp.path("workspaces");
    for name in ["Alpha", "Beta", "Gamma", "Delta", "Epsilon"] {
        let path = gui_workspace_project_path(&projects_dir, name).expect("project path");
        save_gui_workspace_project(
            &path,
            &GuiWorkspaceProject {
                name: name.to_string(),
                files: vec![file_path.clone()],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");
    }
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_manager(&mut runtime);

    let manager = runtime
        .workspace_manager
        .as_ref()
        .expect("workspace manager");
    assert_eq!(manager.entries.len(), 5);
    assert_eq!(manager.entries[0].name, "Alpha");
    assert!(runtime.status.contains("Workspace manager"));
}

#[test]
fn tui_workspace_manager_enter_opens_selected_project() {
    let temp = TempArea::new("tui-workspace-manager-open");
    let alpha_path = temp.path("alpha.txt");
    let beta_path = temp.path("beta.txt");
    let original_path = temp.path("original.txt");
    fs::write(&alpha_path, "alpha\n").expect("write alpha");
    fs::write(&beta_path, "beta\n").expect("write beta");
    fs::write(&original_path, "original\n").expect("write original");
    let projects_dir = temp.path("workspaces");
    for (name, path) in [("Alpha", alpha_path), ("Beta", beta_path.clone())] {
        save_gui_workspace_project(
            &gui_workspace_project_path(&projects_dir, name).expect("project path"),
            &GuiWorkspaceProject {
                name: name.to_string(),
                files: vec![path],
                active_ordinal: 0,
                layout: None,
            },
        )
        .expect("save project");
    }
    let mut original = open_text_file(&original_path).expect("open original");
    let mut workspace = EditorWorkspace::from_document(&mut original);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_manager(&mut runtime);
    handle_workspace_manager_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    handle_workspace_manager_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(runtime.workspace_manager, None);
    assert_eq!(workspace.active_tab().document.as_ref().path, beta_path);
    assert_eq!(runtime.status, "Opened workspace: Beta");
}

#[test]
fn tui_workspace_manager_s_saves_over_selected_project() {
    let temp = TempArea::new("tui-workspace-manager-save-over");
    let old_path = temp.path("old.txt");
    let new_path = temp.path("new.txt");
    fs::write(&old_path, "old\n").expect("write old");
    fs::write(&new_path, "new\n").expect("write new");
    let projects_dir = temp.path("workspaces");
    save_gui_workspace_project(
        &gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
        &GuiWorkspaceProject {
            name: "Alpha".to_string(),
            files: vec![old_path],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save alpha");
    let mut document = open_text_file(&new_path).expect("open new");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir.clone()),
        ..EditorRuntime::default()
    };

    open_workspace_manager(&mut runtime);
    handle_workspace_manager_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
    );

    let project = parse_gui_workspace_project(
        &fs::read_to_string(
            gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
        )
        .expect("read alpha"),
    )
    .expect("parse alpha");
    assert_eq!(project.files, vec![new_path]);
    assert_eq!(runtime.workspace_manager, None);
    assert_eq!(runtime.status, "Workspace saved: Alpha");
}

#[test]
fn tui_workspace_manager_d_starts_delete_confirmation() {
    let temp = TempArea::new("tui-workspace-manager-delete");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "note\n").expect("write note");
    let projects_dir = temp.path("workspaces");
    let project_path = gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "Alpha".to_string(),
            files: vec![file_path.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save alpha");
    let mut document = open_text_file(&file_path).expect("open note");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_manager(&mut runtime);
    handle_workspace_manager_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
    );

    assert_eq!(runtime.workspace_manager, None);
    assert_eq!(
        runtime.workspace_pending_delete,
        Some(("Alpha".to_string(), project_path))
    );
    assert_eq!(
        runtime.workspace_prompt,
        Some(WorkspacePrompt::ConfirmDelete)
    );
}
