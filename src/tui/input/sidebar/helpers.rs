pub(crate) fn validated_sidebar_child_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(String::from("Name is empty"));
    }
    if name == "." || name == ".." {
        return Err(String::from("Name cannot be . or .."));
    }
    if name.starts_with('.') {
        return Err(String::from(
            "Hidden names are not created from the sidebar",
        ));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(String::from("Name must be local, not a path"));
    }
    if name.chars().any(char::is_control) {
        return Err(String::from("Name contains a control character"));
    }
    Ok(name)
}

pub(crate) fn sidebar_target_directory(runtime: &EditorRuntime) -> Option<PathBuf> {
    let sidebar = runtime.sidebar.as_ref()?;
    let selected = sidebar.selected_entry();
    match selected.map(|entry| entry.kind) {
        Some(FileSidebarEntryKind::Directory) => selected.map(|entry| entry.path.clone()),
        Some(FileSidebarEntryKind::Parent | FileSidebarEntryKind::File) | None => {
            Some(sidebar.current_dir.clone())
        }
    }
}

pub(crate) fn refresh_sidebar_after_path_in_dir(
    runtime: &mut EditorRuntime,
    directory: &Path,
    path: &Path,
) {
    match FileSidebarState::load(directory.to_path_buf()) {
        Ok(mut refreshed) => {
            if let Some(index) = refreshed
                .entries
                .iter()
                .position(|entry| entry.path == path)
            {
                refreshed.selected = index;
                refreshed.keep_selection_visible(runtime.page_rows);
            }
            runtime.last_sidebar_dir = Some(refreshed.current_dir.clone());
            runtime.sidebar = Some(refreshed);
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

pub(crate) fn refresh_sidebar_after_delete(runtime: &mut EditorRuntime) {
    let Some(sidebar) = runtime.sidebar.as_ref() else {
        return;
    };
    let current_dir = sidebar.current_dir.clone();
    let old_selected = sidebar.selected;
    match FileSidebarState::load(current_dir) {
        Ok(mut refreshed) => {
            if !refreshed.entries.is_empty() {
                refreshed.selected = old_selected.min(refreshed.entries.len() - 1);
                refreshed.keep_selection_visible(runtime.page_rows);
            }
            runtime.last_sidebar_dir = Some(refreshed.current_dir.clone());
            runtime.sidebar = Some(refreshed);
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

pub(crate) fn open_dirty_tab_uses_path(workspace: &EditorWorkspace<'_>, path: &Path) -> bool {
    workspace.tabs.iter().any(|tab| {
        let document = tab.document.as_ref();
        document.path == path && document.buffer.is_dirty()
    })
}
