#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceManagerState {
    pub(crate) entries: Vec<WorkspaceManagerEntry>,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceManagerEntry {
    pub(crate) name: String,
    pub(crate) files: usize,
}
