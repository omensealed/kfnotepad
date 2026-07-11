pub(super) fn resolve_browser_child_path(
    base_directory: &Path,
    raw_name: &str,
) -> Result<PathBuf, String> {
    let name = raw_name.trim();
    if name.is_empty() {
        return Err("name required".to_string());
    }
    let relative = Path::new(name);
    if relative.is_absolute() {
        return Err("absolute paths are not allowed here".to_string());
    }
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("parent paths are not allowed here".to_string());
    }
    let Some(file_name) = relative.file_name().and_then(|name| name.to_str()) else {
        return Err("valid UTF-8 name required".to_string());
    };
    if file_name == "." || file_name.is_empty() {
        return Err("valid name required".to_string());
    }

    Ok(base_directory.join(relative))
}
