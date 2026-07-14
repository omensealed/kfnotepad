use super::*;

#[test]
fn gui_menu_surface_lists_primary_command_groups() {
    assert!(gui_menu_uses_iced_aw_menu_tree());
    assert_eq!(
            gui_menu_submenu_policy(),
            "Keep current root command groups flat until a group gains enough depth to justify nested hover submenus."
        );
    assert_eq!(
        gui_menu_groups().map(gui_menu_group_label),
        ["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
    );
    assert_eq!(
        gui_menu_groups()
            .map(gui_menu_group_chrome_label)
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["File", "Edit", "View", "Nav", "Notes", "Tile", "Help"]
    );
    assert_eq!(
        gui_menu_dropdown_labels(GuiMenuGroup::File),
        vec![
            LABEL_NEW_TILE,
            LABEL_OPEN,
            LABEL_OPEN_PATH,
            LABEL_SAVE,
            LABEL_SAVE_AS,
            LABEL_SAVE_AS_PATH,
            LABEL_CLOSE_TILE,
            LABEL_QUIT,
        ]
    );
    assert_eq!(
        gui_menu_dropdown_labels(GuiMenuGroup::Edit),
        vec![
            LABEL_UNDO,
            LABEL_REDO,
            LABEL_COPY,
            LABEL_CUT,
            LABEL_PASTE,
            LABEL_SELECT_ALL,
            LABEL_FIND_NEXT,
            LABEL_FIND_PREVIOUS,
        ]
    );
    assert_eq!(gui_menu_group_index(GuiMenuGroup::File), 0);
    assert_eq!(gui_menu_group_index(GuiMenuGroup::Tile), 5);
    assert_eq!(gui_menu_group_index(GuiMenuGroup::Help), 6);

    let file_commands: Vec<_> = gui_menu_items(GuiMenuGroup::File)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        file_commands,
        vec![
            GuiMenuCommand::NewTile,
            GuiMenuCommand::Open,
            GuiMenuCommand::OpenPath,
            GuiMenuCommand::Save,
            GuiMenuCommand::SaveAs,
            GuiMenuCommand::SaveAsPath,
            GuiMenuCommand::ClosePane,
            GuiMenuCommand::Quit,
        ]
    );

    let edit_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Edit)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        edit_commands,
        vec![
            GuiMenuCommand::Undo,
            GuiMenuCommand::Redo,
            GuiMenuCommand::Copy,
            GuiMenuCommand::Cut,
            GuiMenuCommand::Paste,
            GuiMenuCommand::SelectAll,
            GuiMenuCommand::FindNext,
            GuiMenuCommand::FindPrevious,
        ]
    );

    let go_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Go)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        go_commands,
        vec![
            GuiMenuCommand::GoToLine,
            GuiMenuCommand::GoDocumentStart,
            GuiMenuCommand::GoDocumentEnd,
        ]
    );

    let notes_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Notes)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        notes_commands,
        vec![
            GuiMenuCommand::OpenManagedNote,
            GuiMenuCommand::ListManagedNotes,
        ]
    );

    let tile_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Tile)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(
        tile_commands,
        vec![
            GuiMenuCommand::ToggleMinimize,
            GuiMenuCommand::ToggleMaximize,
            GuiMenuCommand::EqualizeTiles,
            GuiMenuCommand::MoveLeft,
            GuiMenuCommand::MoveRight,
            GuiMenuCommand::MoveUp,
            GuiMenuCommand::MoveDown,
        ]
    );

    let help_commands: Vec<_> = gui_menu_items(GuiMenuGroup::Help)
        .into_iter()
        .map(|item| item.command)
        .collect();
    assert_eq!(help_commands, vec![GuiMenuCommand::OpenHelp]);
}

#[test]
fn gui_help_menu_opens_builtin_help_document_tile() {
    let temp = TempArea::new("gui-help-menu");
    let mut state = KfnotepadGui::new_with_current_dir(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
    );

    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));

    let active = state.workspace.active_tile();
    assert_eq!(active.document.path, temp.path(GUI_HELP_DOCUMENT_PATH));
    let help_text = active.document.buffer.to_text();
    assert!(help_text.contains("# kfnotepad help"));
    assert!(help_text.contains("Double-click a file to open it in a tile."));
    assert!(help_text.contains("Ctrl-R, View > Reader mode"));
    assert!(help_text.contains("Search is case-insensitive by default."));
    assert!(help_text.contains("Ctrl-Shift-T cycles the syntax highlighting theme"));
    assert!(help_text.contains("Reader speed is configured in Preferences"));
    assert!(help_text.contains("Notes > Open note creates or opens a managed Markdown note."));
    assert!(help_text.contains("Tile > Equalize tiles arranges open tiles into an even grid."));
    assert!(help_text.contains("Opening a file that is already open focuses or restores"));
    assert!(help_text.contains("Save uses the same atomic local-file adapter"));
    assert!(!active.document.buffer.is_dirty());
    assert!(state.status_message.contains("opened help"));

    let tile_count = state.workspace.tiles.len();
    let help_path = active.document.path.clone();
    let _ = update(&mut state, Message::MenuCommand(GuiMenuCommand::OpenHelp));
    assert_eq!(state.workspace.tiles.len(), tile_count);
    assert_eq!(state.workspace.active_tile().document.path, help_path);
}

#[test]
fn gui_menu_styles_use_app_theme_palette() {
    let palette = gui_theme_palette(EditorThemeId::Nocturne);
    let panel_style = gui_menu_panel_style(palette);

    assert_eq!(
        panel_style.menu_border.radius.top_left,
        GUI_MENU_DROPDOWN_RADIUS
    );
    assert_eq!(panel_style.menu_border.color, palette.primary);
    assert_eq!(panel_style.menu_shadow.offset, Vector::new(0.0, 6.0));
    assert_eq!(panel_style.menu_shadow.blur_radius, 16.0);
    assert_eq!(
        panel_style.path_border.radius.top_left,
        GUI_MENU_ITEM_RADIUS
    );

    let active = gui_menu_item_button_style(palette, iced::widget::button::Status::Active);
    assert_eq!(active.text_color, palette.text);
    assert_eq!(active.border.width, 0.0);
    assert_eq!(active.border.radius.top_left, GUI_MENU_ITEM_RADIUS);

    let root = gui_menu_root_style(palette);
    assert_eq!(root.text_color, Some(palette.text));
    assert!(root.background.is_none());
    assert_eq!(GUI_MENU_ROOT_HORIZONTAL_PADDING, 3.0);
    assert_eq!(GUI_MENU_ROOT_VERTICAL_PADDING, 1.0);
    assert_eq!(GUI_MENU_ROOT_HEIGHT, 24.0);
    assert_eq!(GUI_MENU_BAR_SPACING, 1);
    assert_eq!(GUI_HEADER_ACTION_SPACING, 3);
    assert_eq!(GUI_HEADER_GROUP_SPACING, 6);
    assert_eq!(GUI_HEADER_SPLIT_SPACING, 3);
    assert_eq!(GUI_MENU_ITEM_PADDING, [3, 5]);
    assert_eq!(gui_icon_font(), iced_fonts::NERD_FONT);
    assert_eq!(GUI_ICON_LINE_HEIGHT, 1.0);
    assert_eq!(GUI_PANEL_PADDING_LEFT, 2.0);
    assert_eq!(GUI_PANEL_PADDING_RIGHT, 4.0);
    assert_eq!(GUI_PANEL_PADDING_VERTICAL, 6.0);
    assert_eq!(GUI_EDITOR_RENDER_LINE_BUDGET, 512);

    let hovered = gui_menu_item_button_style(palette, iced::widget::button::Status::Hovered);
    assert_eq!(hovered.text_color, palette.background);
    assert_eq!(hovered.border.width, 1.0);
    assert_eq!(hovered.border.color, palette.primary);

    let chrome = gui_chrome_button_style(palette, iced::widget::button::Status::Active);
    assert_eq!(chrome.text_color, palette.background);
    assert_eq!(chrome.border.width, 0.0);
    assert_eq!(chrome.border.radius.top_left, 4.0);
}

#[test]
fn gui_form_styles_do_not_add_hover_borders() {
    let palette = gui_theme_palette(EditorThemeId::Nocturne);

    let input_active = gui_text_input_style(palette, iced::widget::text_input::Status::Active);
    let input_hovered = gui_text_input_style(palette, iced::widget::text_input::Status::Hovered);
    assert_eq!(input_hovered.border, input_active.border);
    assert_eq!(input_active.value, palette.text);

    let checkbox_active = gui_checkbox_style(
        palette,
        iced::widget::checkbox::Status::Active { is_checked: true },
    );
    assert_eq!(checkbox_active.text_color, Some(palette.text));
    assert_eq!(checkbox_active.border.color, palette.primary);
}

#[test]
fn gui_chrome_labels_trim_paths_and_keep_tooltip_sources() {
    let path = PathBuf::from("/tmp/kfnotepad/deep/example.md");

    assert_eq!(gui_file_name_label(&path), "example.md");
    assert_eq!(
        gui_tile_title_label(&path, true, "modified"),
        "active | example.md | modified"
    );
    assert_eq!(
        gui_tile_title_label(&path, true, "saved"),
        "active | example.md"
    );
    assert_eq!(gui_tile_title_label(&path, false, "saved"), "example.md");
    assert_eq!(
        gui_icon_label(ICON_FILES, LABEL_FILES),
        format!("{ICON_FILES} Files")
    );
    assert_eq!(gui_icon_only_label(ICON_FILES), ICON_FILES);
    assert!(!gui_icon_only_label(ICON_PREFERENCES).contains(LABEL_PREFERENCES));
    assert_eq!(gui_icon_only_label(ICON_NEW_TILE), ICON_NEW_TILE);
    assert!(!gui_icon_only_label(ICON_NEW_TILE).contains(LABEL_NEW_TILE));
    assert_eq!(gui_icon_only_label(ICON_SAVE), ICON_SAVE);
    assert!(!gui_icon_only_label(ICON_SAVE).contains(LABEL_SAVE));
    assert_eq!(gui_icon_only_label(ICON_THEME), ICON_THEME);
    assert!(!gui_icon_only_label(ICON_THEME).contains(LABEL_THEME));
    assert!(!gui_tile_title_label(&path, true, "modified").contains("/tmp"));
    assert!(gui_tile_title_controls_attached(true));
    assert!(gui_tile_title_controls_attached(false));

    let deep_path = PathBuf::from("/home/example/projects/kfnotepad/docs");
    assert_eq!(
        gui_sidebar_path_label(&path),
        "/tmp/kfnotepad/deep/example.md"
    );
    assert_eq!(gui_sidebar_path_label(&deep_path), ".../docs");
    assert!(gui_sidebar_path_label(&deep_path).len() <= GUI_PANEL_PATH_MAX_CHARS);
}
