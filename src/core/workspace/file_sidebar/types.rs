#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSidebarState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileSidebarEntry>,
    pub selected: usize,
    pub scroll: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSidebarEntry {
    pub label: String,
    pub path: PathBuf,
    pub kind: FileSidebarEntryKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSidebarEntryKind {
    Parent,
    Directory,
    File,
}

#[derive(Debug)]
pub enum FileSidebarError {
    ReadDir { path: PathBuf, source: io::Error },
}

impl fmt::Display for FileSidebarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDir { path, source } => {
                write!(formatter, "cannot list {}: {source}", path.display())
            }
        }
    }
}
