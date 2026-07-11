#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GuiFileTreeRowModel {
    pub(super) path: PathBuf,
    label: String,
    kind: FileSidebarEntryKind,
    depth: usize,
    expanded: bool,
    pub(super) selected: bool,
    error: bool,
}

#[cfg(test)]
impl GuiFileTreeRowModel {
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

    pub(super) const fn error(&self) -> bool {
        self.error
    }
}
