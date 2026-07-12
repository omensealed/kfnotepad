//! Public managed-note and metadata snapshot value types.

use super::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadataSnapshot {
    pub bytes: u64,
    pub modified: Option<SystemTime>,
}

impl FileMetadataSnapshot {
    pub fn matches_file_snapshot(&self, snapshot: &FileSnapshot) -> bool {
        self.bytes == snapshot.bytes && self.modified == snapshot.modified
    }
}
