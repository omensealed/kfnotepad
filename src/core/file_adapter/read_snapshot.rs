//! Conservative file reads, save-target validation, and external-change snapshots.

use super::*;

#[derive(Debug)]
pub(super) enum BoundedFileReadError {
    Access(io::Error),
    Directory,
    Symlink,
    NotRegular,
    TooLarge { bytes: u64 },
}

struct BoundedFileRead {
    bytes: Vec<u8>,
    modified: Option<SystemTime>,
}

fn read_bounded_bytes<R: Read>(
    reader: &mut R,
    size_hint: u64,
) -> Result<Vec<u8>, BoundedFileReadError> {
    let sentinel_limit = MAX_TEXT_FILE_BYTES + 1;
    let capacity =
        usize::try_from(size_hint.min(sentinel_limit)).expect("text file limit fits in usize");
    let mut bytes = Vec::with_capacity(capacity);
    reader
        .take(sentinel_limit)
        .read_to_end(&mut bytes)
        .map_err(BoundedFileReadError::Access)?;
    if bytes.len() as u64 > MAX_TEXT_FILE_BYTES {
        return Err(BoundedFileReadError::TooLarge {
            bytes: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

fn read_bounded_regular_file(path: &Path) -> Result<BoundedFileRead, BoundedFileReadError> {
    let path_metadata = fs::symlink_metadata(path).map_err(BoundedFileReadError::Access)?;
    if path_metadata.file_type().is_symlink() {
        return Err(BoundedFileReadError::Symlink);
    }
    if path_metadata.is_dir() {
        return Err(BoundedFileReadError::Directory);
    }
    if !path_metadata.file_type().is_file() {
        return Err(BoundedFileReadError::NotRegular);
    }

    let mut file = File::open(path).map_err(BoundedFileReadError::Access)?;
    let metadata = file.metadata().map_err(BoundedFileReadError::Access)?;
    if metadata.is_dir() {
        return Err(BoundedFileReadError::Directory);
    }
    if !metadata.file_type().is_file() {
        return Err(BoundedFileReadError::NotRegular);
    }
    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(BoundedFileReadError::TooLarge {
            bytes: metadata.len(),
        });
    }

    let bytes = read_bounded_bytes(&mut file, metadata.len())?;
    let modified = file
        .metadata()
        .map_err(BoundedFileReadError::Access)?
        .modified()
        .ok();
    Ok(BoundedFileRead { bytes, modified })
}

fn bounded_read_error(error: BoundedFileReadError) -> io::Error {
    match error {
        BoundedFileReadError::Access(error) => error,
        BoundedFileReadError::Directory => {
            io::Error::new(io::ErrorKind::IsADirectory, "path is a directory")
        }
        BoundedFileReadError::Symlink => {
            io::Error::new(io::ErrorKind::InvalidInput, "path is a symlink")
        }
        BoundedFileReadError::NotRegular => {
            io::Error::new(io::ErrorKind::InvalidInput, "path is not a regular file")
        }
        BoundedFileReadError::TooLarge { bytes } => io::Error::new(
            io::ErrorKind::FileTooLarge,
            format!("text file is too large: {bytes} bytes exceeds {MAX_TEXT_FILE_BYTES} bytes"),
        ),
    }
}

pub(super) fn read_text_file(path: &Path) -> Result<String, OpenError> {
    read_text_file_with_snapshot(path).map(|(text, _snapshot)| text)
}

pub(super) fn read_text_file_with_snapshot(
    path: &Path,
) -> Result<(String, FileSnapshot), OpenError> {
    let bounded = read_bounded_regular_file(path).map_err(|error| match error {
        BoundedFileReadError::Access(source) => OpenError::Access {
            path: path.to_path_buf(),
            source,
        },
        BoundedFileReadError::Directory => OpenError::Directory {
            path: path.to_path_buf(),
        },
        BoundedFileReadError::Symlink => OpenError::Symlink {
            path: path.to_path_buf(),
        },
        BoundedFileReadError::NotRegular => OpenError::NotRegular {
            path: path.to_path_buf(),
        },
        BoundedFileReadError::TooLarge { bytes } => OpenError::TooLarge {
            path: path.to_path_buf(),
            bytes,
            limit: MAX_TEXT_FILE_BYTES,
        },
    })?;
    let text = String::from_utf8(bounded.bytes).map_err(|source| OpenError::ReadUtf8 {
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidData, source),
    })?;
    let snapshot = FileSnapshot {
        bytes: text.len() as u64,
        modified: bounded.modified,
        fingerprint: fingerprint_bytes(text.as_bytes()),
    };
    Ok((text, snapshot))
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

pub(super) fn file_snapshot(path: &Path) -> Result<FileSnapshot, BoundedFileReadError> {
    let bounded = read_bounded_regular_file(path)?;
    Ok(FileSnapshot {
        bytes: bounded.bytes.len() as u64,
        modified: bounded.modified,
        fingerprint: fingerprint_bytes(&bounded.bytes),
    })
}

pub fn snapshot_text_file(path: &Path) -> io::Result<Option<FileSnapshot>> {
    match read_bounded_regular_file(path) {
        Ok(bounded) => Ok(Some(FileSnapshot {
            bytes: bounded.bytes.len() as u64,
            modified: bounded.modified,
            fingerprint: fingerprint_bytes(&bounded.bytes),
        })),
        Err(BoundedFileReadError::Access(error)) if error.kind() == io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(BoundedFileReadError::Directory)
        | Err(BoundedFileReadError::Symlink)
        | Err(BoundedFileReadError::NotRegular) => Ok(None),
        Err(error) => Err(bounded_read_error(error)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_stream_rejects_the_first_byte_past_the_limit() {
        let mut stream = io::repeat(b'x');

        let error = read_bounded_bytes(&mut stream, 0).expect_err("sentinel byte must fail");

        assert!(matches!(
            error,
            BoundedFileReadError::TooLarge { bytes }
                if bytes == MAX_TEXT_FILE_BYTES + 1
        ));
    }

    #[test]
    fn strong_file_snapshot_rejects_oversized_sparse_file() {
        let path = std::env::temp_dir().join(format!(
            "kfnotepad-file-snapshot-oversized-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let file = File::create(&path).expect("create sparse fixture");
        file.set_len(MAX_TEXT_FILE_BYTES + 1)
            .expect("size sparse fixture");
        drop(file);

        let error = file_snapshot(&path).expect_err("oversized snapshot must fail");

        let _ = fs::remove_file(&path);
        assert!(matches!(
            error,
            BoundedFileReadError::TooLarge { bytes }
                if bytes == MAX_TEXT_FILE_BYTES + 1
        ));
    }
}
