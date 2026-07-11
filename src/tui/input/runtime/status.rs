//! Active-tab and path status formatting.

use super::*;

pub(crate) fn active_tab_status(workspace: &EditorWorkspace<'_>) -> String {
    let tab = workspace.active_tab();
    format!(
        "Tab {}/{}: {}",
        workspace.active + 1,
        workspace.tabs.len(),
        display_file_name(&tab.document.as_ref().path)
    )
}

pub(crate) fn display_file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled")
}
