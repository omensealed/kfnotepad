impl KfnotepadGui {
pub(super) fn path_is_open_in_workspace(&self, path: &Path) -> bool {
        self.workspace
            .tiles
            .iter()
            .any(|tile| gui_paths_refer_to_same_file(&tile.document.path, path))
    }

    pub(super) fn directory_contains_open_tile(&self, directory: &Path) -> bool {
        let canonical_directory = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf());
        self.workspace.tiles.iter().any(|tile| {
            tile.document
                .path
                .canonicalize()
                .unwrap_or_else(|_| tile.document.path.clone())
                .starts_with(&canonical_directory)
    })
}
}
