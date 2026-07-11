pub(super) fn delete_browser_path(path: &Path, kind: FileSidebarEntryKind) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "refusing to delete symlink",
        ));
    }
    match kind {
        FileSidebarEntryKind::File if metadata.is_file() => move_path_to_trash(path),
        FileSidebarEntryKind::Directory if metadata.is_dir() => move_path_to_trash(path),
        FileSidebarEntryKind::Parent => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "cannot delete parent shortcut",
        )),
        FileSidebarEntryKind::File => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected path is not a file",
        )),
        FileSidebarEntryKind::Directory => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected path is not a directory",
        )),
    }
}
