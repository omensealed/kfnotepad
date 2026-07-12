//! Symlink-skipping, directory-first sidebar listing and sorting.

use super::*;

pub fn list_file_sidebar_entries(
    current_dir: &Path,
) -> Result<Vec<FileSidebarEntry>, FileSidebarError> {
    let mut directories = Vec::new();
    let mut files = Vec::new();

    if let Some(parent) = current_dir.parent() {
        directories.push(FileSidebarEntry {
            label: String::from("../"),
            path: parent.to_path_buf(),
            kind: FileSidebarEntryKind::Parent,
        });
    }

    let entries = fs::read_dir(current_dir).map_err(|source| FileSidebarError::ReadDir {
        path: current_dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if file_type.is_dir() {
            directories.push(FileSidebarEntry {
                label: format!("{name}/"),
                path: entry.path(),
                kind: FileSidebarEntryKind::Directory,
            });
        } else if file_type.is_file() {
            files.push(FileSidebarEntry {
                label: name,
                path: entry.path(),
                kind: FileSidebarEntryKind::File,
            });
        }
    }

    let sort_start = usize::from(
        directories
            .first()
            .is_some_and(|entry| entry.kind == FileSidebarEntryKind::Parent),
    );
    directories[sort_start..].sort_by_key(|entry| entry.label.to_lowercase());
    files.sort_by_key(|entry| entry.label.to_lowercase());
    directories.extend(files);
    Ok(directories)
}
