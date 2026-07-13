use super::*;

#[test]
fn gui_launch_loads_requested_file_into_editor_state() {
    let temp = TempArea::new("gui-open");
    let path = temp.path("note.txt");
    fs::write(&path, "alpha\nbeta\n").expect("write file");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![path.clone()],
    });

    assert_eq!(state.workspace.active_tile().document.path, path);
    assert_eq!(state.active_editor().text(), "alpha\nbeta\n");
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );
}

#[test]
fn gui_launches_multiple_requested_files_as_tiles_and_panes() {
    let temp = TempArea::new("gui-multi-open");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.active_editor().text(), "second\n");
    assert!(state.panes.iter().any(|(_pane, pane_state)| state
        .workspace
        .tile(pane_state.tile_id)
        .is_some_and(|tile| tile.document.path == first)));
}

#[test]
fn gui_launch_shows_startup_help_panel_without_requested_files() {
    let temp = TempArea::new("gui-startup-help");
    let state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    assert!(state.show_startup_help_panel);
}

#[test]
fn gui_launch_hides_startup_help_panel_when_files_are_requested() {
    let temp = TempArea::new("gui-startup-help-with-file");
    let first = temp.path("first.txt");
    fs::write(&first, "first\n").expect("write file");

    let state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone()],
        },
        temp.root.clone(),
    );

    assert!(!state.show_startup_help_panel);
}

#[test]
fn gui_startup_help_panel_can_be_dismissed() {
    let temp = TempArea::new("gui-startup-help-dismiss");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    assert!(state.show_startup_help_panel);
    let _ = update(&mut state, Message::DismissStartupHelp);
    assert!(!state.show_startup_help_panel);
}

#[test]
fn gui_launch_uses_balanced_halloy_style_split_axes_for_three_files() {
    let temp = TempArea::new("gui-balanced-launch");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");

    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone(), third.clone()],
    });

    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = state.panes.layout()
    else {
        panic!("expected root split");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(first));
    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = &**b
    else {
        panic!("expected active-pane nested split");
    };
    assert_eq!(*axis, pane_grid::Axis::Horizontal);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(second));
    assert_eq!(node_path(&state, b), Some(third));
}

#[test]
fn gui_file_open_splits_the_active_tall_pane_horizontally() {
    let temp = TempArea::new("gui-balanced-open");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });

    state.open_path_in_new_pane(third.clone());

    let pane_grid::Node::Split { axis, b, .. } = state.panes.layout() else {
        panic!("expected root split");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = &**b
    else {
        panic!("expected new file split inside active pane");
    };
    assert_eq!(*axis, pane_grid::Axis::Horizontal);
    assert_eq!(*ratio, 0.5);
    assert_eq!(node_path(&state, a), Some(second));
    assert_eq!(node_path(&state, b), Some(third.clone()));
    assert_eq!(state.workspace.active_tile().document.path, third);
    assert!(state.status_message.starts_with("opened "));
}

#[test]
fn gui_new_tile_splits_active_pane_without_creating_a_file() {
    let temp = TempArea::new("gui-new-tile");
    let existing_untitled = temp.path("untitled.txt");
    fs::write(&existing_untitled, "already here\n").expect("write existing untitled");
    let first = temp.path("first.txt");
    fs::write(&first, "first\n").expect("write first");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone()],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::NewTileRequested);

    let expected = temp.path("untitled-2.txt");
    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, expected);
    assert_eq!(state.active_editor().text(), "");
    assert!(!temp.path("untitled-2.txt").exists());
    assert_eq!(
        fs::read_to_string(&existing_untitled).expect("read existing untitled"),
        "already here\n"
    );
    assert!(state.status_message.starts_with("new tile "));
}

#[test]
fn gui_browser_width_updates_with_clamped_bounds() {
    let temp = TempArea::new("gui-browser-width");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::BrowserWidthChanged(190.0));
    assert_eq!(state.browser_width, 190.0);
    assert_eq!(state.status_message, "file browser width: 190px");

    let _ = update(&mut state, Message::BrowserWidthChanged(999.0));
    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MAX);

    let _ = update(&mut state, Message::BrowserWidthChanged(1.0));
    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MIN);
}
