//! Containment-checked, symlink-safe managed-note deletion through OS trash.

use super::*;

pub fn delete_managed_note(
    notes_dir: &Path,
    note_path: &Path,
) -> Result<ManagedNoteDeleteResult, ManagedNotesError> {
    if !is_managed_note_file_name(note_path) {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path must be a visible .md file".to_string(),
        });
    }

    let Some(note_parent) = note_path.parent() else {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path has no parent directory".to_string(),
        });
    };

    let canonical_notes_dir = match notes_dir.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::ListNotesDir {
                path: notes_dir.to_path_buf(),
                source,
            });
        }
    };

    let canonical_note_parent = match note_parent.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::InspectNote {
                path: note_parent.to_path_buf(),
                source,
            });
        }
    };

    if canonical_note_parent != canonical_notes_dir {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path is outside the notes directory".to_string(),
        });
    }

    let metadata = match fs::symlink_metadata(note_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ManagedNoteDeleteResult::Missing);
        }
        Err(source) => {
            return Err(ManagedNotesError::InspectNote {
                path: note_path.to_path_buf(),
                source,
            });
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "refusing to delete a symlinked managed note".to_string(),
        });
    }
    if !file_type.is_file() {
        return Err(ManagedNotesError::InvalidNotePath {
            path: note_path.to_path_buf(),
            message: "managed note path is not a normal file".to_string(),
        });
    }

    match move_path_to_trash(note_path) {
        Ok(()) => Ok(ManagedNoteDeleteResult::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(ManagedNoteDeleteResult::Missing)
        }
        Err(source) => Err(ManagedNotesError::RemoveNote {
            path: note_path.to_path_buf(),
            source,
        }),
    }
}
