use super::*;

#[test]
fn gui_close_only_clean_pane_resets_to_blank_tile() {
    let temp = TempArea::new("gui-close-only");
    let file = temp.path("only.txt");
    fs::write(&file, "only\n").expect("write only");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(
        state.workspace.active_tile().document.path,
        temp.path("untitled.txt")
    );
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(
        fs::read_to_string(&file).expect("original file unchanged"),
        "only\n"
    );
    assert!(state.status_message.starts_with("new blank tile "));
}

#[test]
fn gui_close_only_dirty_pane_requires_confirmation_before_blank_reset() {
    let temp = TempArea::new("gui-close-only-dirty");
    let file = temp.path("only.txt");
    fs::write(&file, "only\n").expect("write only");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("dirty\n");

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
    assert_eq!(
        state.status_message,
        "unsaved changes; close again to discard this tile"
    );

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(
        state.workspace.active_tile().document.path,
        temp.path("untitled.txt")
    );
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(
        fs::read_to_string(&file).expect("original file unchanged"),
        "only\n"
    );
}

#[test]
fn gui_minimize_active_pane_hides_tile_and_focuses_visible_fallback() {
    let temp = TempArea::new("gui-minimize-active");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    let second_pane = state.active_pane;
    let second_tile_id = state.panes.get(second_pane).expect("second pane").tile_id;
    state
        .panes
        .get_mut(second_pane)
        .expect("second pane")
        .editor = GuiEditorAdapter::from_text("dirty second\n");

    let _ = update(&mut state, Message::ToggleActiveMinimize);

    assert!(
        state
            .workspace
            .tile(second_tile_id)
            .expect("second tile")
            .minimized
    );
    assert_eq!(
        state
            .workspace
            .tile(second_tile_id)
            .expect("second tile")
            .document
            .buffer
            .to_text(),
        "dirty second\n"
    );
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert_eq!(state.status_message, "minimized tile");
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.minimized_panes.len(), 1);
    assert_eq!(state.minimized_panes[0].tile_id, second_tile_id);

    let _ = update(&mut state, Message::RestoreMinimizedTile(second_tile_id));

    assert!(
        !state
            .workspace
            .tile(second_tile_id)
            .expect("second tile")
            .minimized
    );
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.active_editor().text(), "dirty second\n");
    assert_eq!(state.status_message, "restored tile");
    assert_eq!(state.panes.len(), 2);
    assert!(state.minimized_panes.is_empty());
}

#[test]
fn gui_minimize_refuses_only_visible_tile() {
    let temp = TempArea::new("gui-minimize-only-visible");
    let file = temp.path("only.txt");
    fs::write(&file, "only\n").expect("write only");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file],
    });
    let active_tile_id = state.workspace.active;

    let _ = update(&mut state, Message::ToggleActiveMinimize);

    assert!(
        !state
            .workspace
            .tile(active_tile_id)
            .expect("active tile")
            .minimized
    );
    assert_eq!(
        state.status_message,
        "cannot minimize the only visible tile"
    );
}

#[test]
fn gui_close_last_visible_tile_promotes_minimized_tile() {
    let temp = TempArea::new("gui-close-last-visible-promotes-minimized");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    let second_tile_id = state
        .panes
        .get(state.active_pane)
        .expect("second pane")
        .tile_id;

    let _ = update(&mut state, Message::ToggleActiveMinimize);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.minimized_panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, first);

    let _ = update(&mut state, Message::CloseActivePane);

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert!(state.minimized_panes.is_empty());
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert!(
        !state
            .workspace
            .tile(second_tile_id)
            .expect("second tile")
            .minimized
    );
    assert_eq!(state.active_editor().text(), "second\n");
    assert_eq!(state.status_message, format!("closed {}", first.display()));
}

#[test]
fn gui_maximize_active_pane_toggles_without_changing_saved_geometry() {
    let temp = TempArea::new("gui-maximize-active");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    let active = state.active_pane;

    let _ = update(&mut state, Message::ToggleActiveMaximize);

    assert_eq!(state.panes.maximized(), Some(active));
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.status_message, "maximized tile");
    let pane_grid::Node::Split { axis, ratio, .. } = state.panes.layout() else {
        panic!("expected split layout to remain available");
    };
    assert_eq!(*axis, pane_grid::Axis::Vertical);
    assert_eq!(*ratio, 0.5);

    let _ = update(&mut state, Message::ToggleMaximizePane(active));

    assert_eq!(state.panes.maximized(), None);
    assert_eq!(state.status_message, "restored tile layout");
}

#[test]
fn gui_maximized_focus_moves_to_clicked_or_opened_pane() {
    let temp = TempArea::new("gui-maximize-focus");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let third = temp.path("third.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    fs::write(&third, "third\n").expect("write third");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    let first_pane = pane_for_path(&state, &first);

    let _ = update(&mut state, Message::ToggleActiveMaximize);
    assert_eq!(state.panes.maximized(), Some(state.active_pane));

    let _ = update(&mut state, Message::PaneClicked(first_pane));

    assert_eq!(state.active_pane, first_pane);
    assert_eq!(state.panes.maximized(), Some(first_pane));
    assert_eq!(state.workspace.active_tile().document.path, first);

    state.open_path_in_new_pane(third.clone());

    assert_eq!(state.panes.maximized(), Some(state.active_pane));
    assert_eq!(state.workspace.active_tile().document.path, third);
}

#[test]
fn gui_menu_tile_can_maximize_and_restore_active_pane() {
    let temp = TempArea::new("gui-maximize-menu");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first, second],
    });
    let active = state.active_pane;

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ToggleMaximize),
    );

    assert_eq!(state.panes.maximized(), Some(active));
    assert_eq!(state.status_message, "maximized tile");

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ToggleMaximize),
    );

    assert_eq!(state.panes.maximized(), None);
    assert_eq!(state.status_message, "restored tile layout");
}

#[test]
fn gui_move_active_pane_swaps_with_adjacent_pane() {
    let temp = TempArea::new("gui-move-active");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    let active = state.active_pane;
    let first_pane = pane_for_path(&state, &first);

    assert_eq!(
        state.panes.adjacent(active, pane_grid::Direction::Left),
        Some(first_pane)
    );

    let _ = update(
        &mut state,
        Message::MoveActivePane(pane_grid::Direction::Left),
    );

    assert_eq!(
        state.panes.adjacent(active, pane_grid::Direction::Right),
        Some(first_pane)
    );
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.status_message, "moved active tile");
}

#[test]
fn gui_drag_drop_moves_pane_to_edge() {
    let temp = TempArea::new("gui-drag-edge");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first, second.clone()],
    });
    let active = state.active_pane;
    let before = pane_x(&state, active);
    assert!(before > 0.0);

    let _ = update(
        &mut state,
        Message::PaneDragged(pane_grid::DragEvent::Dropped {
            pane: active,
            target: pane_grid::Target::Edge(pane_grid::Edge::Left),
        }),
    );

    assert_eq!(pane_x(&state, active), 0.0);
    assert_eq!(state.workspace.active_tile().document.path, second);
    assert_eq!(state.status_message, "moved tile");
}

#[test]
fn gui_resize_updates_pane_grid_split_ratio() {
    let temp = TempArea::new("gui-resize");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first, second],
    });
    let split = *state.panes.layout().splits().next().expect("split");

    let _ = update(
        &mut state,
        Message::PaneResized(pane_grid::ResizeEvent { split, ratio: 0.3 }),
    );

    let pane_grid::Node::Split { ratio, .. } = state.panes.layout() else {
        panic!("expected split layout");
    };
    assert_eq!(*ratio, 0.3);
}
