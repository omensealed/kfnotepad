//! Sidebar visibility, selection, and scrolling controls.

use super::*;

pub(crate) fn toggle_file_sidebar(runtime: &mut EditorRuntime) {
    if runtime.sidebar.is_some() {
        close_file_sidebar(runtime);
        runtime.status = String::from("Files closed");
        return;
    }

    let directory = runtime.last_sidebar_dir.clone().map(Ok).unwrap_or_else(|| {
        env::current_dir().map_err(|error| format!("cannot resolve current directory: {error}"))
    });

    match directory.and_then(|path| FileSidebarState::load(path).map_err(|error| error.to_string()))
    {
        Ok(sidebar) => {
            runtime.last_sidebar_dir = Some(sidebar.current_dir.clone());
            runtime.sidebar = Some(sidebar);
            runtime.status = String::from("Files");
        }
        Err(error) => runtime.status = format!("Files unavailable: {error}"),
    }
}

pub(crate) fn close_file_sidebar(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = runtime.sidebar.take() {
        runtime.last_sidebar_dir = Some(sidebar.current_dir);
    }
    runtime.sidebar_prompt = None;
    runtime.sidebar_query.clear();
}

pub(crate) fn select_previous_sidebar_entry(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = &mut runtime.sidebar {
        sidebar.select_previous_wrapping(runtime.page_rows);
    }
}

pub(crate) fn select_next_sidebar_entry(runtime: &mut EditorRuntime) {
    if let Some(sidebar) = &mut runtime.sidebar {
        sidebar.select_next_wrapping(runtime.page_rows);
    }
}

pub(crate) fn scroll_sidebar_up(runtime: &mut EditorRuntime, visible_rows: usize) -> bool {
    let Some(sidebar) = &mut runtime.sidebar else {
        return false;
    };
    sidebar.scroll_selection_up(visible_rows)
}

pub(crate) fn scroll_sidebar_down(runtime: &mut EditorRuntime, visible_rows: usize) -> bool {
    let Some(sidebar) = &mut runtime.sidebar else {
        return false;
    };
    sidebar.scroll_selection_down(visible_rows)
}
