//! Persisted GUI workspace project value types.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiWorkspaceProject {
    pub name: String,
    pub files: Vec<PathBuf>,
    pub active_ordinal: usize,
    pub layout: Option<GuiLayout>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiWorkspaceProjectEntry {
    pub path: PathBuf,
    pub project: GuiWorkspaceProject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiWorkspaceProjectDeleteResult {
    Deleted,
    Missing,
}
