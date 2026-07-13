use super::*;

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
