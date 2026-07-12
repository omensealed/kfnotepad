//! Conservative file reads, save-target validation, and external-change snapshots.

use super::*;

pub(super) fn read_text_file(path: &Path) -> Result<String, OpenError> {
    read_text_file_with_snapshot(path).map(|(text, _snapshot)| text)
}

pub(super) fn read_text_file_with_snapshot(
    path: &Path,
) -> Result<(String, FileSnapshot), OpenError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| OpenError::Access {
        path: path.to_path_buf(),
        source,
    })?;

    if metadata.file_type().is_symlink() {
        return Err(OpenError::Symlink {
            path: path.to_path_buf(),
        });
    }

    if metadata.is_dir() {
        return Err(OpenError::Directory {
            path: path.to_path_buf(),
        });
    }

    if !metadata.file_type().is_file() {
        return Err(OpenError::NotRegular {
            path: path.to_path_buf(),
        });
    }

    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(OpenError::TooLarge {
            path: path.to_path_buf(),
            bytes: metadata.len(),
            limit: MAX_TEXT_FILE_BYTES,
        });
    }

    fs::read_to_string(path)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::InvalidData {
                OpenError::ReadUtf8 {
                    path: path.to_path_buf(),
                    source,
                }
            } else {
                OpenError::Access {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })
        .map(|text| {
            let snapshot = FileSnapshot {
                bytes: metadata.len(),
                modified: metadata.modified().ok(),
                fingerprint: fingerprint_bytes(text.as_bytes()),
            };
            (text, snapshot)
        })
}

pub(super) fn validate_save_target(path: &Path) -> Result<Option<fs::Permissions>, SaveError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(SaveError::Symlink {
            path: path.to_path_buf(),
        }),
        Ok(metadata) if metadata.is_dir() => Err(SaveError::Directory {
            path: path.to_path_buf(),
        }),
        Ok(metadata) if !metadata.file_type().is_file() => Err(SaveError::NotRegular {
            path: path.to_path_buf(),
        }),
        Ok(metadata) => Ok(Some(metadata.permissions())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(SaveError::Metadata {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub(super) fn file_snapshot(path: &Path) -> io::Result<FileSnapshot> {
    let metadata = fs::symlink_metadata(path)?;
    let bytes = fs::read(path)?;
    Ok(FileSnapshot {
        bytes: metadata.len(),
        modified: metadata.modified().ok(),
        fingerprint: fingerprint_bytes(&bytes),
    })
}

pub fn snapshot_text_file(path: &Path) -> io::Result<Option<FileSnapshot>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                return Ok(None);
            }
            let bytes = fs::read(path)?;
            Ok(Some(FileSnapshot {
                bytes: metadata.len(),
                modified: metadata.modified().ok(),
                fingerprint: fingerprint_bytes(&bytes),
            }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn snapshot_text_file_metadata(path: &Path) -> io::Result<Option<FileMetadataSnapshot>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                return Ok(None);
            }
            Ok(Some(FileMetadataSnapshot {
                bytes: metadata.len(),
                modified: metadata.modified().ok(),
            }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
