//! OS trash integration and atomic-save filesystem helpers.

use super::*;

pub fn move_path_to_trash(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "refusing to delete symlink",
        ));
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected path is not a file or directory",
        ));
    }

    trash::delete(path).map_err(|error| io::Error::other(format!("trash unavailable: {error}")))
}

pub(super) fn fingerprint_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(super) fn write_temp_then_rename(
    target_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), SaveError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(temp_path)
        .map_err(|source| SaveError::CreateTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;

    if let Some(permissions) = permissions {
        file.set_permissions(permissions)
            .map_err(|source| SaveError::CreateTemp {
                path: temp_path.to_path_buf(),
                source,
            })?;
    }

    file.write_all(bytes)
        .map_err(|source| SaveError::WriteTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.sync_all().map_err(|source| SaveError::FlushTemp {
        path: temp_path.to_path_buf(),
        source,
    })?;
    drop(file);

    fs::rename(temp_path, target_path).map_err(|source| SaveError::Rename {
        from: temp_path.to_path_buf(),
        to: target_path.to_path_buf(),
        source,
    })?;
    sync_parent_directory_best_effort(target_path);
    Ok(())
}

pub(super) fn temporary_sibling_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temp_name = format!(".{file_name}.kfnotepad-{nonce}-{}.tmp", std::process::id());
    path.with_file_name(temp_name)
}

fn sync_parent_directory_best_effort(path: &Path) {
    let Some(parent) = path.parent() else {
        return;
    };
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
}

pub(super) fn is_managed_note_file_name(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if file_name.starts_with('.') {
        return false;
    }

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };

    !stem.is_empty() && path.extension().and_then(|extension| extension.to_str()) == Some("md")
}
