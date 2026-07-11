pub fn load_editor_settings(path: &Path) -> Result<EditorSettings, EditorConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(EditorSettings::default());
        }
        Err(source) => {
            return Err(EditorConfigError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    Ok(parse_editor_settings_config(&text))
}
