use super::*;

#[test]
fn gui_core_actions_keep_descriptive_labels_and_access_paths() {
    let actions = gui_action_descriptors();
    assert!(actions.len() >= 14);

    for action in &actions {
        assert!(
            action.label.chars().count() > 1,
            "GUI action label must be descriptive: {}",
            action.label
        );
        assert!(
            action.shortcut.is_some() || action.menu_group.is_some(),
            "GUI action must have keyboard or menu access: {}",
            action.label
        );
    }

    let menu_labels = gui_menu_groups()
        .into_iter()
        .flat_map(gui_menu_items)
        .map(|item| item.label)
        .collect::<Vec<_>>();
    for action in actions.iter().filter(|action| action.menu_group.is_some()) {
        assert!(
            menu_labels.contains(&action.label),
            "menu-backed GUI action missing from menu: {}",
            action.label
        );
    }
}

#[test]
fn gui_focus_order_keeps_global_controls_before_browser_search_and_tiles() {
    let focus_order = gui_focus_order_descriptors(true, false);
    let labels = focus_order
        .iter()
        .map(|step| step.label)
        .collect::<Vec<_>>();

    assert_eq!(
        &labels[..10],
        [
            "File",
            "Edit",
            "View",
            "Nav",
            "Notes",
            "Tile",
            "Help",
            LABEL_NEW_TILE,
            "Hide Files",
            LABEL_THEME,
        ]
    );

    assert!(
        labels
            .iter()
            .position(|label| *label == "File browser entries")
            < labels.iter().position(|label| *label == "Find field")
    );
    assert!(
        labels.iter().position(|label| *label == LABEL_DOCUMENT_END)
            < labels.iter().position(|label| *label == LABEL_MOVE_LEFT)
    );
    assert_eq!(labels.last(), Some(&"Active editor"));

    for step in &focus_order {
        assert!(
            !step.area.trim().is_empty() && !step.label.trim().is_empty(),
            "focus step must have visible context and label"
        );
    }

    for label in [
        "Hide Files",
        LABEL_NEW_TILE,
        LABEL_THEME,
        LABEL_SAVE,
        "Find field",
        LABEL_FIND_PREVIOUS,
        LABEL_FIND_NEXT,
        "Line field",
        LABEL_GO,
        LABEL_DOCUMENT_START,
        LABEL_DOCUMENT_END,
        LABEL_MOVE_LEFT,
        LABEL_MOVE_RIGHT,
        LABEL_MOVE_UP,
        LABEL_MOVE_DOWN,
        LABEL_MINIMIZE,
        LABEL_MAXIMIZE,
        LABEL_CLOSE_TILE,
    ] {
        let step = focus_order
            .iter()
            .find(|step| step.label == label && !matches!(step.area, "menu"))
            .expect("focus step should exist");
        assert!(
            step.keyboard.is_some(),
            "focus step should keep a keyboard equivalent: {label}"
        );
    }
}

#[test]
fn gui_focus_order_reflects_browser_and_minimized_tile_visibility() {
    let browser_hidden = gui_focus_order_descriptors(false, false);
    let hidden_labels = browser_hidden
        .iter()
        .map(|step| step.label)
        .collect::<Vec<_>>();
    assert!(hidden_labels.contains(&"Show Files"));
    assert!(!hidden_labels.contains(&"File browser entries"));
    assert!(hidden_labels.contains(&"Active editor"));

    let minimized = gui_focus_order_descriptors(true, true);
    let minimized_labels = minimized.iter().map(|step| step.label).collect::<Vec<_>>();
    assert!(minimized_labels.contains(&LABEL_RESTORE));
    assert!(!minimized_labels.contains(&LABEL_MINIMIZE));
    assert!(!minimized_labels.contains(&"Active editor"));
}

#[test]
fn gui_menu_file_and_view_commands_route_existing_actions() {
    let temp = TempArea::new("gui-menu-file-view");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    let config = temp.path("config.toml");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "second\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second.clone()],
    });
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };
    state
        .panes
        .get_mut(state.active_pane)
        .expect("active pane")
        .editor = GuiEditorAdapter::from_text("saved by menu\n");
    let save_tile_id = state.workspace.active_tile().id;

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Save));
    let source_revision = state
        .workspace
        .active_tile()
        .document
        .buffer
        .edit_revision();
    fs::write(&second, "saved by menu\n").expect("simulate async menu save");
    let snapshot = gui_file_snapshot(&second)
        .expect("snapshot menu save")
        .expect("menu save file");
    let _ = update(
        &mut state,
        Message::SaveActiveTileCompleted {
            tile_id: save_tile_id,
            result: Ok(GuiSaveResult {
                source_revision,
                snapshot,
            }),
        },
    );

    assert_eq!(
        fs::read_to_string(&second).expect("read second"),
        "saved by menu\n"
    );

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ToggleBrowser),
    );
    assert!(!state.browser_visible);

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::CycleTheme));
    assert_eq!(state.settings.theme_id, EditorThemeId::Abyss);
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::ClosePane));
    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, first);
}

#[test]
fn gui_file_menu_new_tile_routes_to_new_tile_creation() {
    let temp = TempArea::new("gui-menu-new-tile");
    let first = temp.path("first.txt");
    fs::write(&first, "first\n").expect("write first");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::NewTile));

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(
        state.workspace.active_tile().document.path,
        temp.path("untitled.txt")
    );
    assert!(!temp.path("untitled.txt").exists());
}

#[test]
fn gui_menu_edit_go_and_tile_commands_route_existing_actions() {
    let temp = TempArea::new("gui-menu-edit-go-tile");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first");
    fs::write(&second, "alpha\nbeta alpha\n").expect("write second");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![first.clone(), second],
    });

    let _ = update(&mut state, Message::SearchQueryChanged("alpha".to_string()));
    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::FindNext));
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 5 }
    );

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::GoDocumentStart),
    );
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );

    let second_pane = state.active_pane;
    let second_tile = state.panes.get(second_pane).expect("second pane").tile_id;
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::ToggleMinimize),
    );
    assert!(
        state
            .workspace
            .tile(second_tile)
            .expect("second tile")
            .minimized
    );
    assert_eq!(state.workspace.active_tile().document.path, first);
}

#[test]
fn gui_menu_undo_redo_route_replacement_editor_history() {
    let temp = TempArea::new("gui-menu-undo-redo");
    let file = temp.path("edit.txt");
    fs::write(&file, "hello\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );
    assert_eq!(state.active_editor().text(), "Xhello\n");

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Undo));
    assert_eq!(state.active_editor().text(), "hello\n");
    assert_eq!(state.status_message, "undo");

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Redo));
    assert_eq!(state.active_editor().text(), "Xhello\n");
    assert_eq!(state.status_message, "redo");
}

#[test]
fn gui_insert_key_toggles_overwrite_for_replacement_editor() {
    let temp = TempArea::new("gui-insert-overwrite");
    let file = temp.path("edit.txt");
    fs::write(&file, "abc\n").expect("write file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![file],
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::ToggleReplacementOverwriteMode);
    assert!(state.replacement_overwrite_mode);
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('X')]),
    );
    assert_eq!(state.active_editor().text(), "Xbc\n");

    let _ = update(&mut state, Message::ToggleReplacementOverwriteMode);
    assert!(!state.replacement_overwrite_mode);
    let _ = update(
        &mut state,
        Message::ReplacementEditorInputs(vec![GuiEditorReplacementInput::InsertChar('Y')]),
    );
    assert_eq!(state.active_editor().text(), "XYbc\n");
}

#[test]
fn gui_menu_clipboard_commands_route_editor_actions() {
    let temp = TempArea::new("gui-menu-clipboard");
    let file = temp.path("clip.txt");
    fs::write(&file, "alpha\n").expect("write file");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file.clone()],
    });

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Copy));
    assert_eq!(state.status_message, "nothing selected");
    assert_eq!(
        fs::read_to_string(&file).expect("read file"),
        "alpha\n",
        "clipboard menu actions should not write files"
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::SelectAll));
    assert_eq!(state.status_message, "selected all");
    assert_eq!(
        state.active_editor().selection().as_deref(),
        Some("alpha\n")
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Copy));
    assert_eq!(state.status_message, "copied selection");
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "alpha\n"
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::Cut));
    assert_eq!(state.status_message, "cut selection");
    assert_eq!(state.active_editor().text(), "");
    assert_eq!(state.workspace.active_tile().document.buffer.to_text(), "");
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Modified
    );
    assert_eq!(
        fs::read_to_string(&file).expect("read file after cut"),
        "alpha\n",
        "cut should not save implicitly"
    );

    let _ = update(
        &mut state,
        Message::ClipboardPasted(Some("omega".to_string())),
    );
    assert_eq!(state.status_message, "pasted clipboard");
    assert_eq!(state.active_editor().text(), "omega");
    assert_eq!(
        state.workspace.active_tile().document.buffer.to_text(),
        "omega"
    );

    let _ = update(&mut state, Message::ClipboardPasted(None));
    assert_eq!(state.status_message, "clipboard is empty");
    assert_eq!(state.active_editor().text(), "omega");
}

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

#[test]
fn gui_theme_palettes_cover_existing_presets() {
    let pastel = gui_theme_palette(EditorThemeId::Paper);
    assert_eq!(pastel.background, color(245, 226, 244));
    assert_eq!(pastel.text, color(34, 24, 48));

    let terminal = gui_theme_palette(EditorThemeId::Terminal);
    assert_eq!(terminal.background, color(0, 18, 7));
    assert_eq!(terminal.primary, color(72, 255, 112));

    let abyss = gui_theme_palette(EditorThemeId::Abyss);
    assert_eq!(abyss.background, color(3, 7, 18));
    assert_eq!(abyss.danger, color(255, 64, 96));

    let terror = gui_theme_palette(EditorThemeId::Terror);
    assert_eq!(terror.background, color(24, 0, 30));
    assert_eq!(terror.primary, color(255, 42, 160));

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        assert_eq!(gui_theme(theme_id).palette(), gui_theme_palette(theme_id));
    }
}

#[test]
fn gui_pastel_syntax_colors_are_darkened_for_readability() {
    let pale = syntect::highlighting::Color {
        r: 220,
        g: 226,
        b: 232,
        a: 255,
    };
    let normal = gui_color_from_syntect(pale, EditorThemeId::Nocturne);
    let pastel = gui_color_from_syntect(pale, EditorThemeId::Paper);

    assert_eq!(normal, Color::from_rgb8(213, 224, 246));
    assert!(pastel.r < normal.r);
    assert!(pastel.g < normal.g);
    assert!(pastel.b < normal.b);
    assert_eq!(pastel, Color::from_rgb8(80, 67, 91));
}

#[test]
fn gui_syntax_theme_colors_keep_readable_contrast() {
    let samples = [
        (220, 226, 232),
        (190, 90, 120),
        (90, 170, 130),
        (120, 140, 230),
    ];

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        let background = gui_color_to_rgb(gui_theme_palette(theme_id).background);
        for (red, green, blue) in samples {
            let foreground = gui_syntax_rgb_for_theme(red, green, blue, theme_id);
            assert!(
                gui_contrast_ratio(foreground, background) >= 4.5,
                "{theme_id:?} foreground {foreground:?} lacks readable contrast on {background:?}"
            );
        }
    }
}

#[test]
fn gui_syntax_theme_role_palettes_stay_varied() {
    let samples = [
        (220, 226, 232),
        (100, 110, 120),
        (220, 80, 120),
        (220, 130, 70),
        (210, 190, 70),
        (80, 190, 120),
        (70, 190, 210),
        (80, 120, 220),
        (170, 95, 230),
    ];

    for theme_id in [
        EditorThemeId::Nocturne,
        EditorThemeId::Aurora,
        EditorThemeId::Paper,
        EditorThemeId::Terminal,
        EditorThemeId::Abyss,
        EditorThemeId::Terror,
    ] {
        let colors = samples
            .into_iter()
            .map(|(red, green, blue)| gui_syntax_rgb_for_theme(red, green, blue, theme_id))
            .collect::<HashSet<_>>();

        assert!(
            colors.len() >= 7,
            "{theme_id:?} syntax palette collapsed into {colors:?}"
        );
    }
}

#[test]
fn gui_highlighter_uses_shared_syntax_tokens_and_preset_theme_mapping() {
    let temp = TempArea::new("gui-syntax-highlight");
    let rust_path = temp.path("main.rs");
    let text_path = temp.path("note.txt");
    fs::write(&rust_path, "fn main() {}\n").expect("write rust");
    fs::write(&text_path, "plain\n").expect("write text");
    let state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![rust_path.clone(), text_path.clone()],
    });
    let rust_tile = state
        .workspace
        .tiles
        .iter()
        .find(|tile| tile.document.path == rust_path)
        .expect("rust tile");
    let text_tile = state
        .workspace
        .tiles
        .iter()
        .find(|tile| tile.document.path == text_path)
        .expect("text tile");

    assert_eq!(
        state
            .syntax_highlighter
            .syntax_token_for_document(&rust_tile.document),
        "rs"
    );
    assert_eq!(
        state
            .syntax_highlighter
            .syntax_token_for_document(&text_tile.document),
        "txt"
    );
    assert_eq!(
        gui_highlighter_theme(EditorThemeId::Paper),
        highlighter::Theme::InspiredGitHub
    );
    assert_eq!(
        gui_highlighter_theme(EditorThemeId::Terror),
        highlighter::Theme::Base16Eighties
    );
}

#[test]
fn gui_theme_cycle_updates_status_and_persists_existing_config_format() {
    let temp = TempArea::new("gui-theme-cycle");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleTheme);

    assert_eq!(state.settings.theme_id, EditorThemeId::Abyss);
    assert_eq!(state.status_message, "theme: abyss");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"abyss\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
fn gui_syntax_theme_cycles_separately_from_app_theme_and_persists() {
    let temp = TempArea::new("gui-syntax-theme-cycle");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        syntax_theme_id: EditorThemeId::Nocturne,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleSyntaxTheme);

    assert_eq!(state.settings.theme_id, EditorThemeId::Terminal);
    assert_eq!(state.settings.syntax_theme_id, EditorThemeId::Aurora);
    assert_eq!(state.status_message, "syntax theme: aurora");
    let saved = fs::read_to_string(&config).expect("read config");
    assert!(saved.contains("theme = \"terminal\"\n"));
    assert!(saved.contains("syntax_theme = \"aurora\"\n"));
}

#[test]
fn gui_search_defaults_case_insensitive_and_can_toggle_sensitive() {
    let temp = TempArea::new("gui-search-case");
    let config = temp.path("config.toml");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "Heading\nAlpha only\n").expect("write note");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![file_path],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        None,
    );
    state.search_query = "alpha".to_string();

    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "found next: alpha");
    assert_eq!(state.workspace.active_tile().state.cursor.row, 1);
    assert_eq!(state.workspace.active_tile().state.cursor.column, 0);

    let _ = update(&mut state, Message::SearchCaseSensitiveChanged(true));
    assert!(state.settings.search_case_sensitive);
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(state.status_message, "no match: alpha");
    assert!(fs::read_to_string(&config)
        .expect("read config")
        .contains("search_case_sensitive = true\n"));
}

#[test]
fn gui_case_insensitive_search_maps_expanded_unicode_to_original_columns() {
    let temp = TempArea::new("gui-search-unicode-case");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "aßb SS\nİstanbul\n").expect("write note");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file_path],
    });

    let _ = update(&mut state, Message::SearchQueryChanged("ss".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 1 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("ß"));

    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 4 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("SS"));

    let _ = update(&mut state, Message::SearchQueryChanged("i".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 1, column: 0 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("İ"));
}

#[test]
fn gui_case_sensitive_search_selects_full_grapheme_for_partial_match() {
    let temp = TempArea::new("gui-search-grapheme-case");
    let file_path = temp.path("note.txt");
    fs::write(&file_path, "🇺🇸 e\u{301}x\n").expect("write note");
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: vec![file_path],
    });

    let _ = update(&mut state, Message::SearchCaseSensitiveChanged(true));
    let _ = update(&mut state, Message::SearchQueryChanged("🇸".to_string()));
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(state.active_editor().selection().as_deref(), Some("🇺🇸"));

    let _ = update(
        &mut state,
        Message::SearchQueryChanged("\u{301}".to_string()),
    );
    let _ = update(&mut state, Message::SearchNext);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 3 }
    );
    assert_eq!(
        state.active_editor().selection().as_deref(),
        Some("e\u{301}")
    );
}

#[test]
fn gui_reader_mode_ticks_scroll_active_document_and_persists_speed() {
    let temp = TempArea::new("gui-reader-mode");
    let config = temp.path("config.toml");
    let file_path = temp.path("long.txt");
    fs::write(
        &file_path,
        (1..=80)
            .map(|line| format!("line {line}\n"))
            .collect::<String>(),
    )
    .expect("write long note");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: vec![file_path],
        },
        temp.root.clone(),
        Some(config.clone()),
        None,
        None,
        None,
    );

    let _ = update(&mut state, Message::ReaderSpeedChanged(120));
    let _ = update(&mut state, Message::ReaderModeChanged(true));
    let before = state
        .panes
        .get(state.active_pane)
        .expect("active pane")
        .editor
        .viewport
        .first_line;
    let _ = update(&mut state, Message::ReaderScrollTick);
    let after = state
        .panes
        .get(state.active_pane)
        .expect("active pane")
        .editor
        .viewport
        .first_line;

    assert!(after > before);
    assert_eq!(
        state.workspace.active_tile().state.cursor,
        DocumentCursor { row: 0, column: 0 }
    );
    assert_eq!(
        state
            .panes
            .get(state.active_pane)
            .expect("active pane")
            .editor
            .document_cursor(),
        DocumentCursor { row: 0, column: 0 }
    );
    assert!(state.settings.gui_reader_mode_enabled);
    assert_eq!(state.settings.gui_reader_lines_per_minute, 120);
    let saved = fs::read_to_string(&config).expect("read config");
    assert!(saved.contains("gui_reader_mode_enabled = true\n"));
    assert!(saved.contains("gui_reader_lines_per_minute = 120\n"));
}

#[test]
fn gui_preferences_panel_toggles_line_numbers_and_wrap_in_config() {
    let temp = TempArea::new("gui-preferences-toggle");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

    assert!(!state.settings.show_line_numbers);
    assert_eq!(state.status_message, "line numbers: off");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::WrapLinesChanged(true));

    assert!(state.settings.wrap_lines);
    assert_eq!(state.status_message, "wrap text: on");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = false\nwrap = true\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"monospace\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );
}

#[test]
fn gui_preferences_panel_cycles_font_family_and_size_in_config() {
    let temp = TempArea::new("gui-preferences-font");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleGuiFontFamily);

    assert_eq!(state.settings.gui_font_family, GuiFontFamily::SansSerif);
    assert_eq!(state.status_message, "font: Sans serif");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 16\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::GuiFontSizeChanged(20));

    assert_eq!(state.settings.gui_font_size, 20);
    assert_eq!(
        state.settings.gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(state.status_message, "editor font size: 20");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 14\n"
        );

    let _ = update(&mut state, Message::GuiUiFontSizeChanged(18));

    assert_eq!(state.settings.gui_font_size, 20);
    assert_eq!(state.settings.gui_ui_font_size, 18);
    assert_eq!(state.status_message, "ui font size: 18");
    assert_eq!(
            fs::read_to_string(&config).expect("read config"),
            "theme = \"terminal\"\nsyntax_theme = \"nocturne\"\nline_numbers = true\nwrap = false\nsearch_case_sensitive = false\ngui_restore_last_workspace = false\ngui_reader_mode_enabled = false\ngui_reader_lines_per_minute = 60\ngui_font_family = \"sans-serif\"\ngui_font_size = 20\ngui_ui_font_size = 18\n"
        );
}

#[test]
fn gui_preferences_panel_rejects_out_of_range_font_size() {
    let temp = TempArea::new("gui-preferences-font-size-bounds");
    let config = temp.path("config.toml");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(config.clone());
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(
        &mut state,
        Message::GuiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
    );

    assert_eq!(state.settings.gui_font_size, DEFAULT_GUI_FONT_SIZE);
    assert_eq!(state.status_message, "editor font size must be 10-32");
    assert!(!config.exists());

    let _ = update(
        &mut state,
        Message::GuiUiFontSizeChanged(MAX_GUI_FONT_SIZE + 1),
    );

    assert_eq!(
        state.settings.gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(state.status_message, "ui font size must be 10-32");
    assert!(!config.exists());
}

#[test]
fn gui_preferences_panel_toggle_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-preferences-toggle-failure");
    let blocked_parent = temp.path("blocked");
    fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(blocked_parent.join("config.toml"));
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::ShowLineNumbersChanged(false));

    assert!(state.settings.show_line_numbers);
    assert!(!state.settings.wrap_lines);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}

#[test]
fn gui_preferences_panel_font_change_rolls_back_on_config_save_failure() {
    let temp = TempArea::new("gui-preferences-font-failure");
    let blocked_parent = temp.path("blocked");
    fs::write(&blocked_parent, "not a directory\n").expect("write blocked parent");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );
    state.config_path = Some(blocked_parent.join("config.toml"));
    state.settings = EditorSettings {
        theme_id: EditorThemeId::Terminal,
        show_line_numbers: true,
        wrap_lines: false,
        gui_restore_last_workspace: false,
        ..EditorSettings::default()
    };

    let _ = update(&mut state, Message::CycleGuiFontFamily);

    assert_eq!(state.settings.gui_font_family, GuiFontFamily::Monospace);
    assert!(state.status_message.starts_with("settings save failed: "));
    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocked parent"),
        "not a directory\n"
    );
}

#[test]
fn gui_editor_font_maps_font_presets() {
    assert_eq!(gui_editor_font(GuiFontFamily::Monospace), Font::MONOSPACE);
    assert_eq!(gui_editor_font(GuiFontFamily::SansSerif), Font::DEFAULT);
    assert_eq!(
        gui_editor_font(GuiFontFamily::Serif).family,
        iced::font::Family::Serif
    );
    assert_eq!(
        gui_editor_font(GuiFontFamily::JetBrainsMono),
        Font::with_name("JetBrains Mono")
    );
    assert_eq!(
        gui_editor_font(GuiFontFamily::FiraCode),
        Font::with_name("Fira Code")
    );
}

#[test]
fn gui_ui_font_size_helpers_use_separate_chrome_setting() {
    let settings = EditorSettings {
        gui_font_size: 30,
        gui_ui_font_size: 18,
        ..EditorSettings::default()
    };

    assert_eq!(
        EditorSettings::default().gui_ui_font_size,
        kfnotepad::DEFAULT_GUI_UI_FONT_SIZE
    );
    assert_eq!(gui_ui_text_size(settings), 18);
    assert_eq!(gui_ui_small_text_size(settings), 16);
    assert_eq!(gui_ui_heading_text_size(settings), 22);
    assert_eq!(gui_ui_icon_text_size(settings), 19);
    assert_eq!(gui_ui_tooltip_text_size(settings), 16);
}

#[test]
fn gui_file_tree_rows_use_gui_ui_font_size() {
    let temp = TempArea::new("gui-file-tree-ui-size");
    fs::create_dir(temp.path("src")).expect("create src");
    fs::write(temp.path("README.md"), "readme\n").expect("write readme");
    let root = temp.root.canonicalize().expect("canonical root");
    let mut expanded = HashSet::new();
    expanded.insert(root.clone());
    let settings = EditorSettings {
        gui_font_size: 30,
        gui_ui_font_size: 19,
        ..EditorSettings::default()
    };

    let rows = gui_file_tree_rows_snapshot(&root, &expanded, None);

    assert_eq!(gui_file_tree_text_size(settings), 19);
    assert_eq!(gui_file_tree_icon_size(settings), 20);
    assert!(rows.iter().any(|row| row.path() == root && row.expanded()));
    assert!(rows
        .iter()
        .any(|row| row.label() == "src" && row.kind() == FileSidebarEntryKind::Directory));
    assert!(rows
        .iter()
        .any(|row| row.label() == "README.md" && row.kind() == FileSidebarEntryKind::File));
}

#[test]
fn gui_file_tree_rows_mark_exact_selected_nested_path() {
    let temp = TempArea::new("gui-file-tree-selected-nested");
    let src = temp.path("src");
    fs::create_dir(&src).expect("create src");
    let nested = src.join("lib.rs");
    fs::write(&nested, "pub fn demo() {}\n").expect("write nested file");
    let root = temp.root.canonicalize().expect("canonical root");
    let src = src.canonicalize().expect("canonical src");
    let nested = nested.canonicalize().expect("canonical nested");
    let mut expanded = HashSet::new();
    expanded.insert(root.clone());
    expanded.insert(src.clone());

    let rows = gui_file_tree_rows_snapshot(&root, &expanded, Some(nested.as_path()));

    assert!(rows
        .iter()
        .any(|row| row.path() == nested && row.selected()));
    assert!(rows.iter().any(|row| row.path() == src && !row.selected()));
}

#[test]
fn gui_file_tree_selected_rows_use_selection_foreground() {
    let palette = gui_theme_palette(EditorThemeId::Abyss);

    assert_eq!(
        gui_file_tree_row_text_color(palette, true, false),
        palette.background
    );
    assert_eq!(
        gui_file_tree_row_text_color(palette, false, false),
        palette.text
    );
    assert_ne!(
        gui_file_tree_row_text_color(palette, true, false),
        palette.text
    );
}

#[test]
fn gui_primary_icons_come_from_nerd_font_symbol_constants() {
    assert_eq!(ICON_VIEW_MENU, nf::cod::COD_EYE);
    assert_eq!(ICON_NEW_TILE, nf::fa::FA_PLUS);
    assert_eq!(ICON_SAVE, nf::cod::COD_SAVE);
    assert_eq!(ICON_FILES, nf::fa::FA_FOLDER);
    assert_eq!(ICON_WORKSPACES, nf::cod::COD_MULTIPLE_WINDOWS);
    assert_eq!(ICON_PREFERENCES, nf::cod::COD_SETTINGS_GEAR);
    assert_eq!(ICON_THEME, nf::cod::COD_SYMBOL_COLOR);
    assert_eq!(ICON_REFRESH, nf::cod::COD_REFRESH);
    assert_eq!(ICON_CREATE_FILE, nf::cod::COD_NEW_FILE);
    assert_eq!(ICON_PARENT_DIR, nf::cod::COD_ARROW_UP);
    assert_eq!(ICON_FIND_PREVIOUS, nf::cod::COD_ARROW_LEFT);
    assert_eq!(ICON_FIND_NEXT, nf::cod::COD_ARROW_RIGHT);
    assert_eq!(ICON_GO_TO_LINE, nf::cod::COD_DEBUG_LINE_BY_LINE);
    assert_eq!(ICON_DOCUMENT_START, nf::oct::OCT_HOME);
    assert_eq!(ICON_DOCUMENT_END, nf::oct::OCT_MOVE_TO_END);
    assert_eq!(ICON_MOVE_LEFT, nf::cod::COD_ARROW_SMALL_LEFT);
    assert_eq!(ICON_MOVE_RIGHT, nf::cod::COD_ARROW_SMALL_RIGHT);
    assert_eq!(ICON_MOVE_UP, nf::cod::COD_ARROW_SMALL_UP);
    assert_eq!(ICON_MOVE_DOWN, nf::cod::COD_ARROW_SMALL_DOWN);
    assert_eq!(ICON_MINIMIZE, nf::fa::FA_WINDOW_MINIMIZE);
    assert_eq!(ICON_RESTORE, nf::fa::FA_WINDOW_RESTORE);
    assert_eq!(ICON_MAXIMIZE, nf::fa::FA_WINDOW_MAXIMIZE);
    assert_eq!(ICON_CLOSE, nf::cod::COD_CHROME_CLOSE);
    assert_eq!(ICON_DELETE, nf::fa::FA_TRASH);
}
