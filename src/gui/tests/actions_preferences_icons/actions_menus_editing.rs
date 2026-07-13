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
