//! Private-permission and durable atomic configuration write helpers.

use super::*;

pub(super) fn write_config_temp_then_rename(
    target_path: &Path,
    temp_path: &Path,
    bytes: &[u8],
) -> Result<(), EditorConfigError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(temp_path)
        .map_err(|source| EditorConfigError::CreateTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes)
        .map_err(|source| EditorConfigError::WriteTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    file.sync_all()
        .map_err(|source| EditorConfigError::FlushTemp {
            path: temp_path.to_path_buf(),
            source,
        })?;
    drop(file);

    fs::rename(temp_path, target_path).map_err(|source| EditorConfigError::Rename {
        from: temp_path.to_path_buf(),
        to: target_path.to_path_buf(),
        source,
    })?;
    sync_parent_directory_best_effort(target_path);
    Ok(())
}

fn sync_parent_directory_best_effort(path: &Path) {
    let Some(parent) = path.parent() else {
        return;
    };
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
}

#[cfg(unix)]
pub(super) fn set_private_config_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
pub(super) fn set_private_config_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}
