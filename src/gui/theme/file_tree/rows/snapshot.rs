#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GuiFileTreeRowModelSnapshot {
    path: PathBuf,
    label: String,
    kind: FileSidebarEntryKind,
    expanded: bool,
    selected: bool,
}

#[cfg(test)]
impl GuiFileTreeRowModelSnapshot {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn label(&self) -> &str {
        &self.label
    }

    pub(super) fn kind(&self) -> FileSidebarEntryKind {
        self.kind
    }

    pub(super) const fn expanded(&self) -> bool {
        self.expanded
    }

    pub(super) const fn selected(&self) -> bool {
        self.selected
    }
}

#[cfg(test)]
pub(super) fn gui_file_tree_rows_snapshot(
    root: &Path,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
) -> Vec<GuiFileTreeRowModelSnapshot> {
    gui_file_tree_rows(root, expanded_paths, selected_path)
        .into_iter()
        .map(
            |GuiFileTreeRowModel {
                 path,
                 label,
                 kind,
                 expanded,
                 selected,
                 ..
             }| GuiFileTreeRowModelSnapshot {
                path,
                label,
                kind,
                expanded,
                selected,
            },
        )
        .collect()
}
