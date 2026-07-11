#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedNoteEntry {
    pub file_name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedNoteDeleteResult {
    Deleted,
    Missing,
}
