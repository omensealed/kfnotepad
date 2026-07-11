use super::*;
use crate::tui::input::*;
use crate::tui::menu::*;

#[test]
fn file_sidebar_lists_parent_dirs_and_files_in_order() {
    let temp = TempArea::new("sidebar-list");
    fs::create_dir(temp.path("z-dir")).expect("create z dir");
    fs::create_dir(temp.path("a-dir")).expect("create a dir");
    fs::write(temp.path("z.txt"), "z\n").expect("write z file");
    fs::write(temp.path("a.txt"), "a\n").expect("write a file");

    let sidebar = FileSidebarState::load(temp.root.clone()).expect("load sidebar");
    let labels: Vec<_> = sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();

    assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);
}

#[test]
fn file_sidebar_navigates_into_subdirectories_and_parent() {
    let temp = TempArea::new("sidebar-nav");
    fs::create_dir(temp.path("sub")).expect("create sub dir");
    fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write sub file");
    let mut document = TextDocument {
        path: PathBuf::from("note.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut cursor = Cursor { row: 0, column: 0 };
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
    assert_eq!(
        runtime.sidebar.as_ref().expect("sidebar").current_dir,
        temp.path("sub")
            .canonicalize()
            .expect("canonicalize subdirectory")
    );

    runtime.sidebar.as_mut().expect("sidebar").selected = 0;
    activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
    assert_eq!(
        runtime.sidebar.as_ref().expect("sidebar").current_dir,
        temp.root.canonicalize().expect("canonicalize root")
    );
}

#[test]
fn file_sidebar_reopens_in_last_visited_directory() {
    let temp = TempArea::new("sidebar-last-dir");
    fs::create_dir(temp.path("sub")).expect("create sub");
    fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write inside");
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    let sub_dir = temp.path("sub").canonicalize().expect("canonicalize sub");
    assert_eq!(
        runtime.sidebar.as_ref().expect("sidebar").current_dir,
        sub_dir
    );

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
    );
    assert_eq!(runtime.sidebar, None);
    assert_eq!(runtime.last_sidebar_dir, Some(sub_dir.clone()));

    toggle_file_sidebar(&mut runtime);
    assert_eq!(
        runtime.sidebar.as_ref().expect("sidebar").current_dir,
        sub_dir
    );
}

#[test]
fn file_sidebar_opens_clean_file_and_blocks_dirty_switch() {
    let temp = TempArea::new("sidebar-open");
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut cursor = Cursor { row: 0, column: 4 };
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    activate_selected_sidebar_entry(&mut document, &mut cursor, &mut runtime);
    assert_eq!(document.path, next_path);
    assert_eq!(document.buffer.lines(), &["next".to_string()]);
    assert_eq!(cursor, Cursor { row: 0, column: 0 });
    assert_eq!(runtime.sidebar, None);

    let mut dirty_document = TextDocument {
        path: PathBuf::from("dirty.txt"),
        buffer: kfnotepad::TextBuffer::from_text("dirty\n"),
    };
    dirty_document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty document");
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;
    activate_selected_sidebar_entry(&mut dirty_document, &mut cursor, &mut runtime);

    assert_eq!(dirty_document.path, PathBuf::from("dirty.txt"));
    assert_eq!(runtime.status, "Save before opening another file");
    assert!(runtime.sidebar.is_some());
}

#[test]
fn file_sidebar_opens_selected_file_in_new_tab_without_replacing_dirty_current() {
    let temp = TempArea::new("sidebar-open-tab");
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty current document");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        quit_confirmation_pending: true,
        close_tab_confirmation_pending: true,
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    open_selected_sidebar_entry_in_new_tab(&mut workspace, &mut runtime);

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(
        workspace.tabs[0].document.as_ref().path,
        PathBuf::from("current.txt")
    );
    assert!(workspace.tabs[0].document.as_ref().buffer.is_dirty());
    assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
    assert_eq!(
        workspace.active_tab().document.as_ref().buffer.lines(),
        &["next".to_string()]
    );
    assert_eq!(workspace.active_tab().state, EditorTabState::default());
    assert_eq!(runtime.sidebar, None);
    assert_eq!(runtime.status, "Opened tab next.txt");
    assert!(!runtime.quit_confirmation_pending);
    assert!(!runtime.close_tab_confirmation_pending);
}

#[test]
fn sidebar_ctrl_enter_opens_selected_file_in_new_tab() {
    let temp = TempArea::new("sidebar-ctrl-enter-tab");
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    assert!(handle_workspace_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)
    ));

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
    assert_eq!(runtime.status, "Opened tab next.txt");
}

#[test]
fn workspace_sidebar_enter_opens_selected_file_as_visible_tab() {
    let temp = TempArea::new("sidebar-enter-tab");
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let mut document = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty current document");
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(
        workspace.tabs[0].document.as_ref().path,
        PathBuf::from("current.txt")
    );
    assert!(workspace.tabs[0].document.as_ref().buffer.is_dirty());
    assert_eq!(workspace.active_tab().document.as_ref().path, next_path);
    assert_eq!(runtime.sidebar, None);
    assert_eq!(runtime.status, "Opened tab next.txt");
}

#[test]
fn workspace_sidebar_open_focuses_existing_file_tab_without_duplicate() {
    let temp = TempArea::new("sidebar-focus-existing-tab");
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let current = TextDocument {
        path: PathBuf::from("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let next = open_text_file(&next_path).expect("open next");
    let mut workspace = EditorWorkspace {
        tabs: vec![
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(current)),
                state: EditorTabState::default(),
            },
            EditorTab {
                document: EditorTabDocument::Owned(Box::new(next)),
                state: EditorTabState::default(),
            },
        ],
        active: 0,
    };
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active, 1);
    assert_eq!(runtime.status, "Focused tab next.txt");
}

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

#[test]
fn file_sidebar_creates_file_in_selected_directory() {
    let temp = TempArea::new("sidebar-create-file");
    fs::create_dir(temp.path("sub")).expect("create sub dir");
    let mut document = TextDocument {
        path: temp.path("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        page_rows: 20,
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
    );
    for value in "note.txt".chars() {
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
        );
    }
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let created = temp.path("sub").join("note.txt");
    assert!(created.exists());
    assert_eq!(fs::read_to_string(&created).expect("read created file"), "");
    assert_eq!(runtime.status, "Created file note.txt");
    let selected = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .expect("selected entry");
    assert_eq!(selected.path, created);
}

#[test]
fn file_sidebar_creates_directory_and_rejects_path_names() {
    let temp = TempArea::new("sidebar-create-dir");
    let mut document = TextDocument {
        path: temp.path("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        page_rows: 20,
        ..EditorRuntime::default()
    };

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
    );
    for value in "bad/name".chars() {
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
        );
    }
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert_eq!(runtime.status, "Name must be local, not a path");
    assert!(!temp.path("bad").exists());

    runtime.sidebar_query.clear();
    for value in "notes".chars() {
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
        );
    }
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    let created = temp.path("notes");
    assert!(created.is_dir());
    assert_eq!(runtime.status, "Created directory notes/");
    let selected = runtime
        .sidebar
        .as_ref()
        .and_then(FileSidebarState::selected_entry)
        .expect("selected entry");
    assert_eq!(selected.path, created);
}

#[test]
fn file_sidebar_delete_requires_confirmation_and_removes_file() {
    let temp = TempArea::new("sidebar-delete-file");
    let delete_path = temp.path("delete.txt");
    fs::write(&delete_path, "remove\n").expect("write file");
    let mut document = TextDocument {
        path: temp.path("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        page_rows: 20,
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
    );
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );
    assert!(delete_path.exists());
    assert_eq!(runtime.status, "Delete cancelled; type yes to confirm");

    for value in "yes".chars() {
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
        );
    }
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(!delete_path.exists());
    assert_eq!(runtime.status, "Moved to trash delete.txt");
}

#[test]
fn file_sidebar_delete_directory_warns_and_removes_children_after_confirmation() {
    let temp = TempArea::new("sidebar-delete-dir");
    let delete_dir = temp.path("delete-dir");
    fs::create_dir(&delete_dir).expect("create dir");
    fs::write(delete_dir.join("child.txt"), "child\n").expect("write child");
    let mut document = TextDocument {
        path: temp.path("current.txt"),
        buffer: kfnotepad::TextBuffer::from_text("current\n"),
    };
    let mut workspace = EditorWorkspace::from_document(&mut document);
    let mut runtime = EditorRuntime {
        sidebar: Some(FileSidebarState::load(temp.root.clone()).expect("load sidebar")),
        page_rows: 20,
        ..EditorRuntime::default()
    };
    runtime.sidebar.as_mut().expect("sidebar").selected = 1;

    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
    );
    assert_eq!(
        runtime.status,
        "Delete directory and all contents? type yes: "
    );

    for value in "yes".chars() {
        handle_workspace_sidebar_key_event(
            &mut workspace,
            &mut runtime,
            KeyEvent::new(KeyCode::Char(value), KeyModifiers::NONE),
        );
    }
    handle_workspace_sidebar_key_event(
        &mut workspace,
        &mut runtime,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    );

    assert!(!delete_dir.exists());
    assert_eq!(runtime.status, "Moved to trash delete-dir/");
}
