use super::*;

#[test]
fn gui_menu_tile_equalizes_visible_panes_into_grid() {
    let temp = TempArea::new("gui-equalize-menu");
    let paths = (1..=5)
        .map(|index| {
            let path = temp.path(&format!("{index}.txt"));
            fs::write(&path, format!("{index}\n")).expect("write tile");
            path
        })
        .collect::<Vec<_>>();
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: paths.clone(),
    });
    let active_path = state.workspace.active_tile().document.path.clone();

    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
    );

    assert_eq!(state.status_message, "equalized 5 tiles");
    assert_eq!(state.workspace.active_tile().document.path, active_path);
    let Some(layout) = gui_layout_from_state(
        &state.panes,
        &state.workspace,
        state.browser_visible,
        state.browser_width,
    ) else {
        panic!("expected layout");
    };
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = layout.root
    else {
        panic!("expected equalized root split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 666);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
    assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
}

#[test]
fn gui_equalize_preserves_wrapped_editor_viewport_and_gutter() {
    let temp = TempArea::new("gui-equalize-renderer-viewport");
    let paths = (1..=5)
            .map(|index| {
                let path = temp.path(&format!("{index}.txt"));
                let text = (1..=120)
                    .map(|line| {
                        format!(
                            "line {line} keeps enough words around to wrap in a narrow pane without changing the source line"
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(&path, text).expect("write tile");
                path
            })
            .collect::<Vec<_>>();
    let mut state = KfnotepadGui::new(GuiLaunch {
        requested_paths: paths,
    });
    state.settings.show_line_numbers = true;
    state.settings.wrap_lines = true;

    let _ = update(&mut state, Message::ScrollActiveEditorViewport(40));
    let _ = update(
        &mut state,
        Message::MenuCommand(GuiMenuCommand::EqualizeTiles),
    );

    let surface = gui_editor_surface_model(
        state.settings,
        &state.workspace.active_tile().document,
        state.active_editor(),
        &state.syntax_highlighter,
        state.syntax_caches.get(&state.workspace.active_tile().id),
    );
    let line_numbers = surface.line_numbers.expect("visible line numbers");
    let visual_rows = gui_editor_read_only_visual_rows(
        &surface.viewport_slice.lines,
        surface.viewport_slice.first_line,
        surface.wrapping,
        34,
    );

    assert_eq!(surface.viewport_slice.first_line, 41);
    assert_eq!(line_numbers.gutter_start, 41);
    assert!(line_numbers.text.starts_with("41\n42\n43"));
    assert_eq!(visual_rows.first().map(|row| row.line.number), Some(41));
    assert!(visual_rows.iter().any(|row| !row.show_line_number));
    assert!(visual_rows.iter().all(|row| row.line.number >= 41));
    assert_eq!(state.status_message, "equalized 5 tiles");
}

#[test]
fn gui_open_replaces_initial_blank_tile_but_not_dirty_or_real_tiles() {
    let temp = TempArea::new("gui-open-replace-blank");
    let file = temp.path("opened.txt");
    fs::write(&file, "opened\n").expect("write opened file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::OpenDialogSelected(Some(file.clone())));

    assert_eq!(state.workspace.tiles.len(), 1);
    assert_eq!(state.panes.len(), 1);
    assert_eq!(state.workspace.active_tile().document.path, file);
    assert_eq!(state.active_editor().text(), "opened\n");
    assert_eq!(
        state.workspace.active_tile().save_status(),
        GuiTileSaveStatus::Saved
    );

    let second = temp.path("second.txt");
    fs::write(&second, "second\n").expect("write second file");
    let _ = update(
        &mut state,
        Message::OpenDialogSelected(Some(second.clone())),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);
}

#[test]
fn gui_open_focuses_existing_file_instead_of_duplicate_tile() {
    let temp = TempArea::new("gui-open-existing-file");
    let first = temp.path("first.txt");
    let second = temp.path("second.txt");
    fs::write(&first, "first\n").expect("write first file");
    fs::write(&second, "second\n").expect("write second file");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: vec![first.clone(), second.clone()],
        },
        temp.root.clone(),
    );

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, second);

    let _ = update(&mut state, Message::OpenDialogSelected(Some(first.clone())));

    assert_eq!(state.workspace.tiles.len(), 2);
    assert_eq!(state.panes.len(), 2);
    assert_eq!(state.workspace.active_tile().document.path, first);
    assert_eq!(state.active_editor().text(), "first\n");
    assert!(state.status_message.starts_with("already open: "));
}

#[test]
fn gui_tile_title_controls_are_attached_for_hover_reveal_on_every_pane() {
    assert!(gui_tile_title_controls_attached(true));
    assert!(gui_tile_title_controls_attached(false));
}

#[test]
fn gui_header_layout_splits_global_actions_on_narrow_windows() {
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH),
        GuiHeaderLayoutMode::SingleRow
    );
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH + 1.0),
        GuiHeaderLayoutMode::SingleRow
    );
    assert_eq!(
        gui_header_layout_mode(GUI_HEADER_SPLIT_WIDTH - 1.0),
        GuiHeaderLayoutMode::SplitActions
    );
}

#[test]
fn gui_search_layout_splits_controls_on_narrow_windows() {
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH),
        GuiSearchLayoutMode::SingleRow
    );
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH + 1.0),
        GuiSearchLayoutMode::SingleRow
    );
    assert_eq!(
        gui_search_layout_mode(GUI_SEARCH_SPLIT_WIDTH - 1.0),
        GuiSearchLayoutMode::SplitRows
    );
}
