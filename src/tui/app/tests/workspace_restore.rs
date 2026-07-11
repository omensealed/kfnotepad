use super::*;
use crate::tui::input::*;

#[test]
fn tui_restore_autosave_refreshes_current_workspace_after_tab_open() {
    let temp = TempArea::new("tui-autosave-current");
    let first_path = temp.path("first.txt");
    let second_path = temp.path("second.txt");
    fs::write(&first_path, "first\n").expect("write first");
    fs::write(&second_path, "second\n").expect("write second");
    let mut first = open_text_file(&first_path).expect("open first");
    let mut workspace = EditorWorkspace::from_document(&mut first);
    let projects_dir = temp.path("workspaces");
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir.clone()),
        settings: EditorSettings {
            gui_restore_last_workspace: true,
            ..EditorSettings::default()
        },
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 2;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let project_path =
        gui_workspace_project_path(&projects_dir, TUI_CURRENT_WORKSPACE_NAME).expect("path");
    let project = parse_gui_workspace_project(
        &fs::read_to_string(project_path).expect("read saved current workspace"),
    )
    .expect("parse project");
    assert_eq!(project.name, TUI_CURRENT_WORKSPACE_NAME);
    assert_eq!(project.files, vec![first_path, second_path]);
    assert_eq!(project.active_ordinal, 1);
}

#[test]
fn tui_workspace_project_open_ignores_gui_layout_and_replaces_clean_tabs() {
    let temp = TempArea::new("tui-workspace-open-project");
    let first_path = temp.path("first.txt");
    let second_path = temp.path("second.txt");
    let original_path = temp.path("original.txt");
    fs::write(&first_path, "first\n").expect("write first");
    fs::write(&second_path, "second\n").expect("write second");
    fs::write(&original_path, "original\n").expect("write original");
    let projects_dir = temp.path("workspaces");
    let project_path =
        gui_workspace_project_path(&projects_dir, "GUI Project").expect("project path");
    let project = GuiWorkspaceProject {
        name: "GUI Project".to_string(),
        files: vec![first_path.clone(), second_path.clone()],
        active_ordinal: 1,
        layout: Some(kfnotepad::GuiLayout {
            browser_visible: false,
            browser_width_px: Some(220),
            root: kfnotepad::GuiLayoutNode::Split {
                axis: kfnotepad::GuiLayoutAxis::Horizontal,
                ratio_per_mille: 500,
                first: Box::new(kfnotepad::GuiLayoutNode::Leaf { ordinal: 0 }),
                second: Box::new(kfnotepad::GuiLayoutNode::Leaf { ordinal: 1 }),
            },
            minimized_ordinals: vec![0],
        }),
    };
    save_gui_workspace_project(&project_path, &project).expect("save project");
    let mut original = open_text_file(&original_path).expect("open original");
    let mut workspace = EditorWorkspace::from_document(&mut original);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_project_named(&mut workspace, &mut runtime, "GUI Project");

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.tabs[0].document.as_ref().path, first_path);
    assert_eq!(workspace.active_tab().document.as_ref().path, second_path);
    assert_eq!(runtime.status, "Opened workspace: GUI Project");
}

#[test]
fn tui_workspace_project_open_requires_confirmation_for_dirty_tabs() {
    let temp = TempArea::new("tui-workspace-open-dirty");
    let original_path = temp.path("original.txt");
    let project_file = temp.path("project.txt");
    fs::write(&original_path, "original\n").expect("write original");
    fs::write(&project_file, "project\n").expect("write project");
    let projects_dir = temp.path("workspaces");
    let project_path = gui_workspace_project_path(&projects_dir, "Project").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "Project".to_string(),
            files: vec![project_file.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save project");
    let mut original = open_text_file(&original_path).expect("open original");
    original
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty original");
    let mut workspace = EditorWorkspace::from_document(&mut original);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_project_named(&mut workspace, &mut runtime, "Project");

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab().document.as_ref().path, original_path);
    assert_eq!(runtime.workspace_prompt, Some(WorkspacePrompt::ConfirmOpen));
    assert!(runtime.workspace_pending_open.is_some());
    assert!(runtime.workspace_open_confirmation_pending);

    for character in "yes".chars() {
        handle_workspace_prompt_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
        );
    }
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab().document.as_ref().path, project_file);
    assert_eq!(runtime.workspace_prompt, None);
    assert_eq!(runtime.status, "Opened workspace: Project");
}

#[test]
fn tui_workspace_project_open_skips_missing_files_and_loads_available_tabs() {
    let temp = TempArea::new("tui-workspace-open-partial");
    let original_path = temp.path("original.txt");
    let available_path = temp.path("available.txt");
    let missing_path = temp.path("missing.txt");
    fs::write(&original_path, "original\n").expect("write original");
    fs::write(&available_path, "available\n").expect("write available");
    let projects_dir = temp.path("workspaces");
    let project_path = gui_workspace_project_path(&projects_dir, "Partial").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "Partial".to_string(),
            files: vec![missing_path.clone(), available_path.clone()],
            active_ordinal: 1,
            layout: None,
        },
    )
    .expect("save broken project");
    let mut original = open_text_file(&original_path).expect("open original");
    let mut workspace = EditorWorkspace::from_document(&mut original);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_project_named(&mut workspace, &mut runtime, "Partial");

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(
        workspace.active_tab().document.as_ref().path,
        available_path
    );
    assert!(runtime.status.contains("Opened workspace: Partial"));
    assert!(runtime
        .status
        .contains("skipped 1 missing/unavailable file(s)"));
    assert!(runtime.status.contains(&missing_path.display().to_string()));
    assert_ne!(workspace.active_tab().document.as_ref().path, original_path);
}

#[test]
fn tui_workspace_project_open_uses_blank_tab_when_no_files_load() {
    let temp = TempArea::new("tui-workspace-open-all-missing");
    let original_path = temp.path("original.txt");
    let missing_path = temp.path("missing.txt");
    fs::write(&original_path, "original\n").expect("write original");
    let projects_dir = temp.path("workspaces");
    let project_path = gui_workspace_project_path(&projects_dir, "Missing").expect("project path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "Missing".to_string(),
            files: vec![missing_path],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save broken project");
    let mut original = open_text_file(&original_path).expect("open original");
    let mut workspace = EditorWorkspace::from_document(&mut original);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    open_workspace_project_named(&mut workspace, &mut runtime, "Missing");

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(
        workspace.active_tab().document.as_ref().path.file_name(),
        Some(std::ffi::OsStr::new("untitled.txt"))
    );
    assert_eq!(
        workspace.active_tab().document.as_ref().buffer.to_text(),
        ""
    );
    assert!(runtime.status.contains("opened blank tab"));
}
