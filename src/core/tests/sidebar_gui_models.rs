use super::*;

#[test]
fn shared_file_sidebar_lists_parent_dirs_and_files_in_order() {
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
    assert_eq!(sidebar.selected_entry().expect("selected").label, "../");
}

#[test]
fn shared_file_sidebar_loads_subdirectories_and_parent_entries() {
    let temp = TempArea::new("sidebar-nav");
    fs::create_dir(temp.path("sub")).expect("create sub dir");
    fs::write(temp.path("sub").join("inside.txt"), "inside\n").expect("write sub file");

    let sidebar = FileSidebarState::load(temp.root.clone()).expect("load root sidebar");
    let sub = sidebar
        .entries
        .iter()
        .find(|entry| entry.label == "sub/")
        .expect("subdirectory entry")
        .clone();
    assert_eq!(sub.kind, FileSidebarEntryKind::Directory);

    let sub_sidebar = FileSidebarState::load(sub.path).expect("load sub sidebar");
    assert_eq!(
        sub_sidebar.current_dir,
        temp.path("sub")
            .canonicalize()
            .expect("canonicalize subdirectory")
    );
    assert_eq!(
        sub_sidebar.entries.first().expect("parent entry").kind,
        FileSidebarEntryKind::Parent
    );
}

#[test]
fn shared_file_sidebar_selection_wraps_and_scrolls_without_terminal_types() {
    let mut sidebar = FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..5)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 0,
    };

    sidebar.select_previous_wrapping(3);
    assert_eq!(sidebar.selected, 4);
    assert_eq!(sidebar.scroll, 2);

    sidebar.select_next_wrapping(3);
    assert_eq!(sidebar.selected, 0);
    assert_eq!(sidebar.scroll, 0);

    assert!(sidebar.scroll_selection_down(3));
    assert_eq!(sidebar.selected, 1);
    assert_eq!(sidebar.scroll, 0);
    assert!(sidebar.scroll_selection_down(3));
    assert!(sidebar.scroll_selection_down(3));
    assert_eq!(sidebar.selected, 3);
    assert_eq!(sidebar.scroll, 1);
    assert!(sidebar.scroll_selection_up(3));
    assert_eq!(sidebar.selected, 2);
    assert_eq!(sidebar.scroll, 1);

    sidebar.selected = 0;
    sidebar.scroll = 0;
    assert!(!sidebar.scroll_selection_up(3));
    assert_eq!(sidebar.selected, 0);
    assert_eq!(sidebar.scroll, 0);
}

#[test]
fn shared_file_sidebar_mouse_row_selects_visible_entry() {
    let mut sidebar = FileSidebarState {
        current_dir: PathBuf::from("."),
        entries: (0..4)
            .map(|index| FileSidebarEntry {
                label: format!("file-{index}.txt"),
                path: PathBuf::from(format!("file-{index}.txt")),
                kind: FileSidebarEntryKind::File,
            })
            .collect(),
        selected: 0,
        scroll: 1,
    };

    assert_eq!(sidebar.selected_entry_for_mouse_row(0), None);
    assert_eq!(
        sidebar
            .selected_entry_for_mouse_row(2)
            .expect("visible entry")
            .label,
        "file-2.txt"
    );
    assert_eq!(sidebar.selected, 2);
    assert_eq!(sidebar.selected_entry_for_mouse_row(5), None);
    assert_eq!(sidebar.selected, 2);
}

#[test]
fn gui_workspace_opens_two_documents_as_focused_tiles() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);

    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(workspace.focused, GuiTileId(0));
    assert_eq!(
        workspace.active_tile().document.path,
        PathBuf::from("first.txt")
    );

    let second_id = workspace.open_tile(second);

    assert_eq!(second_id, GuiTileId(1));
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(workspace.active, second_id);
    assert_eq!(workspace.focused, second_id);
    assert_eq!(
        workspace.focused_tile().document.path,
        PathBuf::from("second.txt")
    );

    assert!(workspace.focus_tile(GuiTileId(0)));
    assert_eq!(
        workspace.active_tile().document.path,
        PathBuf::from("first.txt")
    );
    assert!(!workspace.focus_tile(GuiTileId(99)));
    assert_eq!(workspace.active, GuiTileId(0));
}

#[test]
fn gui_workspace_blocks_invalid_open_without_mutation() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);

    let result = workspace.open_validated_tile(Err(OpenError::Directory {
        path: PathBuf::from("dir"),
    }));

    assert!(matches!(
        result,
        Err(GuiTileOpenError::Invalid {
            source: OpenError::Directory { .. }
        })
    ));
    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(workspace.focused, GuiTileId(0));
}

#[test]
fn gui_workspace_dirty_close_requires_confirmation() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    second
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty second tile");
    let mut workspace = GuiWorkspace::from_document(first);
    let second_id = workspace.open_tile(second);

    assert_eq!(
        workspace.close_tile(second_id, false),
        GuiCloseTileResult::Dirty { tile_id: second_id }
    );
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(
        workspace.close_tile(second_id, true),
        GuiCloseTileResult::Closed {
            tile_id: second_id,
            path: PathBuf::from("second.txt"),
        }
    );
    assert_eq!(workspace.tiles.len(), 1);
    assert_eq!(workspace.active, GuiTileId(0));
    assert_eq!(
        workspace.close_tile(GuiTileId(0), true),
        GuiCloseTileResult::OnlyTile
    );
}

#[test]
fn gui_workspace_tracks_minimize_and_layout_intents() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let second = TextDocument {
        path: PathBuf::from("second.txt"),
        buffer: TextBuffer::from_text("two\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);
    let second_id = workspace.open_tile(second);

    assert!(workspace.set_tile_minimized(second_id, true));
    assert!(workspace.tile(second_id).expect("tile").minimized);
    assert!(workspace.set_tile_minimized(second_id, false));
    assert_eq!(workspace.focused, second_id);
    assert!(!workspace.set_tile_minimized(GuiTileId(99), true));

    assert!(workspace.request_split(second_id, GuiSplitDirection::Vertical));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Split {
            tile_id: second_id,
            direction: GuiSplitDirection::Vertical,
        })
    );
    assert!(workspace.request_move(second_id, GuiTileMoveDirection::Left));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Move {
            tile_id: second_id,
            direction: GuiTileMoveDirection::Left,
        })
    );
    assert!(workspace.request_resize(second_id, GuiTileResizeDirection::Wider));
    assert_eq!(
        workspace.pending_layout_intent,
        Some(GuiTileLayoutIntent::Resize {
            tile_id: second_id,
            direction: GuiTileResizeDirection::Wider,
        })
    );
    assert!(!workspace.request_split(GuiTileId(99), GuiSplitDirection::Horizontal));
    workspace.clear_layout_intent();
    assert_eq!(workspace.pending_layout_intent, None);
}

#[test]
fn gui_workspace_reports_save_status_from_buffer_and_failures() {
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let mut workspace = GuiWorkspace::from_document(first);
    let tile_id = workspace.active;

    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    workspace
        .active_tile_mut()
        .document
        .buffer
        .insert_char(0, 0, '!')
        .expect("dirty tile");
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Modified
    );

    assert!(workspace.mark_tile_save_failed(tile_id, "permission denied"));
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::SaveFailed {
            message: String::from("permission denied"),
        }
    );
    workspace.active_tile_mut().document.buffer.mark_clean();
    assert!(workspace.clear_tile_save_error(tile_id));
    assert_eq!(
        workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
    assert!(!workspace.mark_tile_save_failed(GuiTileId(99), "missing"));
}

#[test]
fn gui_file_browser_lists_and_navigates_without_iced_types() {
    let temp = TempArea::new("gui-browser-nav");
    fs::create_dir(temp.path("z-dir")).expect("create z dir");
    fs::create_dir(temp.path("a-dir")).expect("create a dir");
    fs::write(temp.path("z.txt"), "z\n").expect("write z file");
    fs::write(temp.path("a.txt"), "a\n").expect("write a file");
    fs::write(temp.path("a-dir").join("inside.txt"), "inside\n").expect("write nested file");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");

    let labels: Vec<_> = browser
        .sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();
    assert_eq!(labels, ["../", "a-dir/", "z-dir/", "a.txt", "z.txt"]);

    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "a-dir/")
        .expect("a-dir entry");
    assert_eq!(
        browser.activate_selected().expect("activate directory"),
        GuiFileBrowserActivation::Navigated {
            current_dir: temp
                .path("a-dir")
                .canonicalize()
                .expect("canonicalize a-dir"),
        }
    );
    assert_eq!(
        browser.selected_entry().expect("parent entry").kind,
        FileSidebarEntryKind::Parent
    );
}

#[test]
fn gui_file_browser_file_activation_opens_new_tile_through_existing_adapter() {
    let temp = TempArea::new("gui-browser-open");
    let first = TextDocument {
        path: PathBuf::from("first.txt"),
        buffer: TextBuffer::from_text("one\n"),
    };
    let next_path = temp.path("next.txt");
    fs::write(&next_path, "next\n").expect("write next file");
    let canonical_next_path = next_path.canonicalize().expect("canonicalize next file");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    let mut workspace = GuiWorkspace::from_document(first);

    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "next.txt")
        .expect("next file entry");

    let activation = browser.activate_selected().expect("activate file");
    assert_eq!(
        activation,
        GuiFileBrowserActivation::OpenTile {
            path: canonical_next_path.clone(),
        }
    );

    let GuiFileBrowserActivation::OpenTile { path } = activation else {
        panic!("expected open tile activation");
    };
    let tile_id = workspace
        .open_validated_tile(open_text_file(&path))
        .expect("open validated tile");
    assert_eq!(tile_id, GuiTileId(1));
    assert_eq!(workspace.tiles.len(), 2);
    assert_eq!(workspace.active_tile().document.path, canonical_next_path);
    assert_eq!(
        workspace.active_tile().document.buffer.lines(),
        &["next".to_string()]
    );
}

#[test]
fn gui_file_browser_refresh_picks_up_external_files_and_preserves_selection() {
    let temp = TempArea::new("gui-browser-refresh");
    fs::write(temp.path("keep.txt"), "keep\n").expect("write keep");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "keep.txt")
        .expect("keep entry");

    fs::write(temp.path("added.txt"), "added\n").expect("write added");

    browser.refresh().expect("refresh browser");

    let labels = browser
        .sidebar
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"added.txt"));
    assert_eq!(
        browser.selected_entry().expect("selected").label,
        "keep.txt"
    );
}

#[test]
fn gui_file_browser_refresh_clamps_selection_when_selected_entry_disappears() {
    let temp = TempArea::new("gui-browser-refresh-clamp");
    let removed = temp.path("removed.txt");
    fs::write(&removed, "removed\n").expect("write removed");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.selected = browser
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "removed.txt")
        .expect("removed entry");

    fs::remove_file(removed).expect("remove selected file");

    browser.refresh().expect("refresh browser");

    assert!(browser.sidebar.selected < browser.sidebar.entries.len());
    assert!(!browser
        .sidebar
        .entries
        .iter()
        .any(|entry| entry.label == "removed.txt"));
}

#[test]
fn gui_file_browser_rejects_invalid_roots_and_empty_selections() {
    let temp = TempArea::new("gui-browser-invalid");
    let missing = temp.path("missing");
    assert!(matches!(
        GuiFileBrowser::load(missing),
        Err(FileSidebarError::ReadDir { .. })
    ));

    let mut browser = GuiFileBrowser {
        sidebar: FileSidebarState {
            current_dir: temp.root.clone(),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
        },
    };
    assert!(matches!(
        browser.activate_selected(),
        Err(GuiFileBrowserError::EmptySelection)
    ));
}

#[test]
fn gui_file_browser_mouse_row_activation_selects_visible_file() {
    let temp = TempArea::new("gui-browser-mouse");
    fs::write(temp.path("first.txt"), "first\n").expect("write first");
    fs::write(temp.path("second.txt"), "second\n").expect("write second");
    let mut browser = GuiFileBrowser::load(temp.root.clone()).expect("load browser");
    browser.sidebar.scroll = 1;

    assert_eq!(browser.activate_mouse_row(0).expect("row zero"), None);
    assert_eq!(
        browser.activate_mouse_row(2).expect("activate row"),
        Some(GuiFileBrowserActivation::OpenTile {
            path: temp
                .path("second.txt")
                .canonicalize()
                .expect("canonicalize second file"),
        })
    );
    assert_eq!(
        browser.selected_entry().expect("selected").label,
        "second.txt"
    );
}
