//! Test-only immutable file-tree row snapshots.

#[cfg(test)]
use super::*;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::gui::app::state) struct GuiFileTreeRowModelSnapshot {
    path: PathBuf,
    label: String,
    kind: FileSidebarEntryKind,
    expanded: bool,
    selected: bool,
}

#[cfg(test)]
impl GuiFileTreeRowModelSnapshot {
    pub(in crate::gui::app::state) fn path(&self) -> &Path {
        &self.path
    }

    pub(in crate::gui::app::state) fn label(&self) -> &str {
        &self.label
    }

    pub(in crate::gui::app::state) fn kind(&self) -> FileSidebarEntryKind {
        self.kind
    }

    pub(in crate::gui::app::state) const fn expanded(&self) -> bool {
        self.expanded
    }

    pub(in crate::gui::app::state) const fn selected(&self) -> bool {
        self.selected
    }
}

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_file_tree_rows_snapshot(
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
