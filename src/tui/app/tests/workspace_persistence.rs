use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;

#[test]
fn help_menu_opens_and_focuses_maintained_help_document() {
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime::default();

    run_workspace_menu_command(MenuCommand::OpenHelp, &mut workspace, &mut runtime);

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    let active = workspace.active_tab().document.as_ref();
    assert_eq!(active.path, PathBuf::from(TUI_HELP_DOCUMENT_PATH));
    assert!(active.buffer.to_text().contains("# kfnotepad help"));
    assert!(active.buffer.to_text().contains("## File sidebar"));
    assert!(active
        .buffer
        .to_text()
        .contains("Ctrl-R toggles reader mode."));
    assert!(active
        .buffer
        .to_text()
        .contains("Ctrl-Shift-T cycles the syntax highlighting theme independently."));
    assert!(active
        .buffer
        .to_text()
        .contains("Managed notes are normal Markdown files."));
    assert!(!active.buffer.is_dirty());
    assert_eq!(runtime.status, "Opened help");

    workspace.active = 0;
    run_workspace_menu_command(MenuCommand::OpenHelp, &mut workspace, &mut runtime);

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Focused help");
}

#[test]
fn tui_workspace_project_save_current_writes_path_only_project() {
    let temp = TempArea::new("tui-workspace-save-current");
    let first_path = temp.path("first.txt");
    let second_path = temp.path("second.txt");
    fs::write(&first_path, "first\n").expect("write first");
    fs::write(&second_path, "second\n").expect("write second");
    let first = open_text_file(&first_path).expect("open first");
    let second = open_text_file(&second_path).expect("open second");
    let mut workspace = EditorWorkspace {
        tabs: vec![
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(first)),
                state: EditorTabState::default(),
            },
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(second)),
                state: EditorTabState::default(),
            },
        ],
        active: 1,
    };
    let projects_dir = temp.path("workspaces");
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir.clone()),
        ..EditorRuntime::default()
    };

    run_workspace_menu_command(
        MenuCommand::SaveCurrentWorkspace,
        &mut workspace,
        &mut runtime,
    );

    let project_path = gui_workspace_project_path(&projects_dir, "current workspace")
        .expect("current project path");
    let project =
        parse_gui_workspace_project(&fs::read_to_string(project_path).expect("read saved project"))
            .expect("parse project");
    assert_eq!(project.name, "current workspace");
    assert_eq!(project.files, vec![first_path, second_path]);
    assert_eq!(project.active_ordinal, 1);
    assert_eq!(project.layout, None);
    assert_eq!(runtime.status, "Workspace saved: current workspace");
}

#[test]
fn tui_workspace_project_save_named_prompt_and_list_projects() {
    let temp = TempArea::new("tui-workspace-save-named");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "note\n").expect("write note");
    let mut document = open_text_file(&file_path).expect("open note");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let projects_dir = temp.path("workspaces");
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir.clone()),
        ..EditorRuntime::default()
    };

    start_workspace_save_prompt(&mut runtime);
    for character in "Project One".chars() {
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

    let project_path =
        gui_workspace_project_path(&projects_dir, "Project One").expect("named project path");
    assert!(project_path.exists());
    assert_eq!(runtime.status, "Workspace saved: Project One");

    open_workspace_manager(&mut runtime);
    let manager = runtime
        .workspace_manager
        .as_ref()
        .expect("workspace manager");
    assert_eq!(manager.entries.len(), 1);
    assert_eq!(manager.entries[0].name, "Project One");
    assert_eq!(
        runtime.status,
        "Workspace manager: Enter open | S save over | D delete | N new | Esc"
    );
}

#[test]
fn tui_new_file_command_creates_clean_untitled_tab_without_writing_file() {
    let temp = TempArea::new("tui-new-file-tab");
    let existing_untitled = temp.path("untitled.txt");
    fs::write(&existing_untitled, "already exists\n").expect("write existing untitled");
    let current_path = temp.path("current.txt");
    fs::write(&current_path, "current\n").expect("write current");
    let mut document = open_text_file(&current_path).expect("open current");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };

    run_workspace_menu_command(MenuCommand::NewFile, &mut workspace, &mut runtime);

    let expected = temp.path("untitled-2.txt");
    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab().document.as_ref().path, expected);
    assert_eq!(
        workspace.active_tab().document.as_ref().buffer.to_text(),
        ""
    );
    assert!(!workspace.active_tab().document.as_ref().buffer.is_dirty());
    assert!(!expected.exists());
    assert_eq!(runtime.status, "New file tab: untitled-2.txt");
}

#[test]
fn tui_workspace_prompt_cycles_existing_names_for_save_overwrite() {
    let temp = TempArea::new("tui-workspace-save-cycle");
    let old_path = temp.path("old.txt");
    let new_path = temp.path("new.txt");
    fs::write(&old_path, "old\n").expect("write old");
    fs::write(&new_path, "new\n").expect("write new");
    let projects_dir = temp.path("workspaces");
    save_gui_workspace_project(
        &gui_workspace_project_path(&projects_dir, "Alpha").expect("alpha path"),
        &GuiWorkspaceProject {
            name: "Alpha".to_string(),
            files: vec![old_path.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save alpha");
    save_gui_workspace_project(
        &gui_workspace_project_path(&projects_dir, "Beta").expect("beta path"),
        &GuiWorkspaceProject {
            name: "Beta".to_string(),
            files: vec![old_path],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save beta");
    let mut document = open_text_file(&new_path).expect("open new");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir.clone()),
        ..EditorRuntime::default()
    };

    start_workspace_save_prompt(&mut runtime);
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    assert_eq!(runtime.workspace_query, "Beta");
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let project = parse_gui_workspace_project(
        &fs::read_to_string(gui_workspace_project_path(&projects_dir, "Beta").expect("beta path"))
            .expect("read beta"),
    )
    .expect("parse beta");
    assert_eq!(project.files, vec![new_path]);
    assert_eq!(runtime.status, "Workspace saved: Beta");
}

#[test]
fn tui_workspace_open_prompt_cycles_saved_projects() {
    let temp = TempArea::new("tui-workspace-open-cycle");
    let alpha_path = temp.path("alpha.txt");
    let beta_path = temp.path("beta.txt");
    let original_path = temp.path("original.txt");
    fs::write(&alpha_path, "alpha\n").expect("write alpha");
    fs::write(&beta_path, "beta\n").expect("write beta");
    fs::write(&original_path, "original\n").expect("write original");
    let projects_dir = temp.path("workspaces");
    for (name, path) in [("Alpha", alpha_path.clone()), ("Beta", beta_path.clone())] {
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

    start_workspace_open_prompt(&mut runtime);
    assert_eq!(runtime.workspace_query, "Alpha");
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
    );
    assert_eq!(runtime.workspace_query, "Beta");
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab().document.as_ref().path, beta_path);
    assert_eq!(runtime.status, "Opened workspace: Beta");
}

#[test]
fn tui_workspace_delete_prompt_requires_confirmation_and_removes_project() {
    let temp = TempArea::new("tui-workspace-delete");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "note\n").expect("write note");
    let projects_dir = temp.path("workspaces");
    let project_path = gui_workspace_project_path(&projects_dir, "Project").expect("path");
    save_gui_workspace_project(
        &project_path,
        &GuiWorkspaceProject {
            name: "Project".to_string(),
            files: vec![file_path.clone()],
            active_ordinal: 0,
            layout: None,
        },
    )
    .expect("save project");
    let mut document = open_text_file(&file_path).expect("open note");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        workspace_projects_dir: Some(projects_dir),
        ..EditorRuntime::default()
    };

    start_workspace_delete_prompt(&mut runtime);
    assert_eq!(runtime.workspace_query, "Project");
    handle_workspace_prompt_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(
        runtime.workspace_prompt,
        Some(WorkspacePrompt::ConfirmDelete)
    );
    assert!(project_path.exists());
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

    assert!(!project_path.exists());
    assert_eq!(runtime.workspace_prompt, None);
    assert_eq!(runtime.status, "Moved workspace to trash: Project");
}
