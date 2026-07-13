//! Browser action-target selection.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui::app::state) fn selected_browser_action_entry(
        &self,
    ) -> Option<FileSidebarEntry> {
        if let Some(path) = self.browser_selected_path.as_deref() {
            if path.is_dir() {
                return Some(FileSidebarEntry {
                    label: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    path: path.to_path_buf(),
                    kind: FileSidebarEntryKind::Directory,
                });
            }
            if path.is_file() {
                return Some(FileSidebarEntry {
                    label: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    path: path.to_path_buf(),
                    kind: FileSidebarEntryKind::File,
                });
            }
        }

        self.browser
            .as_ref()
            .and_then(|browser| browser.selected_entry())
            .cloned()
    }
}
