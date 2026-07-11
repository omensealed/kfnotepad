pub fn open_or_create_managed_note(
    notes_dir: &Path,
    title: &str,
) -> Result<TextDocument, ManagedNotesError> {
    let path = managed_note_path(notes_dir, title)?;

    fs::create_dir_all(notes_dir).map_err(|source| ManagedNotesError::CreateNotesDir {
        path: notes_dir.to_path_buf(),
        source,
    })?;

    if !path.exists() {
        let empty = TextBuffer::from_text("");
        save_text_buffer(&path, &empty).map_err(|source| ManagedNotesError::CreateNote {
            path: path.clone(),
            source,
        })?;
    }

    open_text_file(&path).map_err(|source| ManagedNotesError::OpenNote {
        path: path.clone(),
        source,
    })
}

pub fn list_managed_notes(notes_dir: &Path) -> Result<Vec<ManagedNoteEntry>, ManagedNotesError> {
    let directory = match fs::read_dir(notes_dir) {
        Ok(directory) => directory,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(ManagedNotesError::ListNotesDir {
                path: notes_dir.to_path_buf(),
                source,
            });
        }
    };

    let mut notes = Vec::new();
    for entry in directory {
        let entry = entry.map_err(|source| ManagedNotesError::ListNotesDir {
            path: notes_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| ManagedNotesError::InspectNote {
                path: path.clone(),
                source,
            })?;

        if !file_type.is_file() || !is_managed_note_file_name(&path) {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("validated managed note file name")
            .to_string();
        notes.push(ManagedNoteEntry { file_name, path });
    }

    notes.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(notes)
}
