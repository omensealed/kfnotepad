pub fn managed_notes_dir(
    xdg_data_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, ManagedNotesError> {
    resolve_managed_notes_dir(xdg_data_home, home)
}

pub fn note_slug(title: &str) -> Result<String, ManagedNotesError> {
    let title = title.trim();
    if title.is_empty()
        || title == "."
        || title == ".."
        || title.starts_with('.')
        || title.contains(['/', '\\'])
        || title.chars().any(char::is_control)
    {
        return Err(ManagedNotesError::InvalidNoteName);
    }

    let mut slug = String::new();
    let mut pending_separator = false;
    for character in title.chars() {
        if character.is_alphanumeric() {
            if pending_separator && !slug.is_empty() {
                slug.push('-');
            }
            for lowercase in character.to_lowercase() {
                slug.push(lowercase);
            }
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }

    if slug.is_empty() {
        return Err(ManagedNotesError::InvalidNoteName);
    }

    slug.push_str(".md");
    Ok(slug)
}

pub fn managed_note_path(notes_dir: &Path, title: &str) -> Result<PathBuf, ManagedNotesError> {
    Ok(notes_dir.join(note_slug(title)?))
}
