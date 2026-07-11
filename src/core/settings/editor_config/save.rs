pub fn save_editor_settings(
    path: &Path,
    settings: EditorSettings,
) -> Result<(), EditorConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
        set_private_config_dir_permissions(parent).map_err(|source| {
            EditorConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

    let text = format!(
        "theme = \"{}\"\nsyntax_theme = \"{}\"\nline_numbers = {}\nwrap = {}\nsearch_case_sensitive = {}\ngui_restore_last_workspace = {}\ngui_reader_mode_enabled = {}\ngui_reader_lines_per_minute = {}\ngui_font_family = \"{}\"\ngui_font_size = {}\ngui_ui_font_size = {}\n",
        settings.theme_id.label(),
        settings.syntax_theme_id.label(),
        settings.show_line_numbers,
        settings.wrap_lines,
        settings.search_case_sensitive,
        settings.gui_restore_last_workspace,
        settings.gui_reader_mode_enabled,
        settings.gui_reader_lines_per_minute,
        settings.gui_font_family.label(),
        settings.gui_font_size,
        settings.gui_ui_font_size
    );
    let temp_path = temporary_config_path(path);
    let result = write_config_temp_then_rename(path, &temp_path, text.as_bytes());
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}
