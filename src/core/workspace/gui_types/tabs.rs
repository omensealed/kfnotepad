#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabStripItem {
    pub label: String,
    pub active: bool,
    pub dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseActiveTabResult {
    OnlyTab,
    Dirty,
    Closed { path: PathBuf },
}
