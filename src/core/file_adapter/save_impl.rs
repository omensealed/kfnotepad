fn save_text_buffer_for_document(path: &Path, buffer: &mut TextBuffer) -> Result<(), SaveError> {
    let expected_snapshot = buffer.file_snapshot().cloned();
    let snapshot = save_text_buffer_inner(path, buffer, expected_snapshot.as_ref())?;
    buffer.set_file_snapshot(Some(snapshot));
    Ok(())
}

fn save_text_buffer_inner(
    path: &Path,
    buffer: &TextBuffer,
    expected_snapshot: Option<&FileSnapshot>,
) -> Result<FileSnapshot, SaveError> {
    let text = buffer.to_text();
    if text.len() as u64 > MAX_TEXT_FILE_BYTES {
        return Err(SaveError::TooLarge {
            path: path.to_path_buf(),
            bytes: text.len() as u64,
            limit: MAX_TEXT_FILE_BYTES,
        });
    }

    let existing_permissions = validate_save_target(path)?;
    if let Some(expected_snapshot) = expected_snapshot {
        match file_snapshot(path) {
            Ok(current_snapshot) if current_snapshot != *expected_snapshot => {
                return Err(SaveError::ExternalModification {
                    path: path.to_path_buf(),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(SaveError::ExternalModification {
                    path: path.to_path_buf(),
                });
            }
            Err(source) => {
                return Err(SaveError::Metadata {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
    }

    let temp_path = temporary_sibling_path(path);
    let save_result =
        write_temp_then_rename(path, &temp_path, text.as_bytes(), existing_permissions);

    if save_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    save_result?;
    file_snapshot(path).map_err(|source| SaveError::Metadata {
        path: path.to_path_buf(),
        source,
    })
}
