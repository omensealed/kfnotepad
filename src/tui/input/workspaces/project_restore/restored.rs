//! Validated restored workspace state.

use super::*;

pub(crate) struct RestoredTuiWorkspace {
    pub(crate) project_name: String,
    pub(crate) workspace: EditorWorkspace<'static>,
    pub(crate) skipped_files: Vec<String>,
    pub(crate) created_blank: bool,
}

impl RestoredTuiWorkspace {
    pub(crate) fn status_message(&self) -> Option<String> {
        if self.skipped_files.is_empty() {
            return Some(format!("Opened workspace: {}", self.project_name));
        }

        let first = self
            .skipped_files
            .first()
            .map(String::as_str)
            .unwrap_or("unknown path");
        let loaded = if self.created_blank {
            "opened blank tab".to_string()
        } else {
            format!("loaded {} file(s)", self.workspace.tabs.len())
        };
        Some(format!(
            "Opened workspace: {}; skipped {} missing/unavailable file(s), {loaded}; first: {first}",
            self.project_name,
            self.skipped_files.len()
        ))
    }
}
