use super::*;

#[test]
fn gui_restores_valid_layout_without_storing_paths() {
    let temp = TempArea::new("gui-layout-restore");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
    fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = false\nroot = 0\nnode.0 = split horizontal 250 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\nminimized = 0\n",
        )
        .expect("write layout");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second],
        },
        temp.root.clone(),
        None,
        Some(layout_path),
        None,
        None,
    );

    assert!(!state.browser_visible);
    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_DEFAULT);
    assert!(
        state
            .workspace
            .tile(GuiTileId(0))
            .expect("first tile")
            .minimized
    );
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.minimized_panes.len(), 1);
    assert_eq!(state.minimized_panes[0].tile_id, GuiTileId(0));
    assert!(matches!(state.panes.layout(), pane_grid::Node::Pane(_)));
    assert!(state.status_message.contains("restored GUI layout"));
}

#[test]
fn gui_restores_and_clamps_persisted_browser_width() {
    let temp = TempArea::new("gui-layout-browser-width");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
    fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = true\nbrowser_width_px = 999\nroot = 0\nnode.0 = split vertical 500 1 2\nnode.1 = leaf 0\nnode.2 = leaf 1\nminimized =\n",
        )
        .expect("write layout");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first, second],
        },
        temp.root.clone(),
        None,
        Some(layout_path),
        None,
        None,
    );

    assert_eq!(state.browser_width, GUI_BROWSER_WIDTH_MAX);
    assert!(state.status_message.contains("restored GUI layout"));
}

#[test]
fn gui_restores_nested_layout_for_three_panes() {
    let temp = TempArea::new("gui-layout-restore-nested");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");
    fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
    fs::write(
            &layout_path,
            "version = 1\nbrowser_visible = true\nroot = 0\nnode.0 = split horizontal 400 1 2\nnode.1 = leaf 0\nnode.2 = split vertical 700 3 4\nnode.3 = leaf 1\nnode.4 = leaf 2\nminimized =\n",
        )
        .expect("write layout");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone(), third.clone()],
        },
        temp.root.clone(),
        None,
        Some(layout_path),
        None,
        None,
    );

    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = state.panes.layout()
    else {
        panic!("expected restored root split");
    };
    assert_eq!(*axis, pane_grid::Axis::Horizontal);
    assert_eq!(*ratio, 0.4);
    assert_eq!(node_path(&state, a), Some(first));
    let pane_grid::Node::Split {
        axis, ratio, a, b, ..
    } = &**b
    else {
        panic!("expected nested split");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    assert_eq!(*ratio, 0.7);
    assert_eq!(node_path(&state, a), Some(second));
    assert_eq!(node_path(&state, b), Some(third));
}

#[test]
fn gui_ignores_invalid_layout_and_uses_default_launch_layout() {
    let temp = TempArea::new("gui-layout-invalid");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::create_dir_all(layout_path.parent().expect("layout parent")).expect("layout dir");
    fs::write(&layout_path, "version = 99\nroot = 0\nnode.0 = leaf 0\n")
        .expect("write invalid layout");

    let state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second],
        },
        temp.root.clone(),
        None,
        Some(layout_path),
        None,
        None,
    );

    assert!(state.browser_visible);
    let pane_grid::Node::Split { axis, ratio, .. } = state.panes.layout() else {
        panic!("expected default split layout");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    assert_eq!(*ratio, 0.5);
    assert!(!state.status_message.contains("restored GUI layout"));
    assert_eq!(pane_x(&state, pane_for_path(&state, &first)), 0.0);
}

#[test]
fn gui_saves_geometry_only_layout_after_layout_changes() {
    let temp = TempArea::new("gui-layout-save-runtime");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let layout_path = temp.path("config").join("kfnotepad").join("gui-layout.v1");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
        None,
        Some(layout_path.clone()),
        None,
        None,
    );
    let split = *state.panes.layout().splits().next().expect("split");

    let _ = update(
        &mut state,
        Message::PaneResized(pane_grid::ResizeEvent { split, ratio: 0.3 }),
    );
    let _ = update(&mut state, Message::ToggleBrowser);
    let _ = update(&mut state, Message::BrowserWidthChanged(270.0));
    let _ = update(&mut state, Message::ToggleActiveMinimize);

    let text = fs::read_to_string(&layout_path).expect("read saved layout");
    let layout = parse_gui_layout(&text, 2).expect("parse saved layout");
    assert!(!layout.browser_visible);
    assert_eq!(layout.browser_width_px, Some(270));
    assert_eq!(layout.minimized_ordinals, vec![1]);
    assert!(text.contains("node.0 = split vertical 500"));
    assert!(text.contains("browser_width_px = 270"));
    assert!(!text.contains(first.to_string_lossy().as_ref()));
    assert!(!text.contains(second.to_string_lossy().as_ref()));
    assert!(!text.contains("first\n"));
    assert!(!text.contains("second\n"));
}
