use super::*;
use crate::tui::input::*;

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
