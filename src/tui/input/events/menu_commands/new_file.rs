//! New untitled workspace-tab creation.

use super::*;

pub(crate) fn create_new_file_tab(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let path = next_tui_untitled_path(workspace, runtime);
    let document = TextDocument {
        path: path.clone(),
        buffer: kfnotepad::TextBuffer::from_text(""),
    };
    workspace.push_owned_tab(document);
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    stop_reader_mode(runtime, "Reader mode stopped for new file");
    runtime.status = format!("New file tab: {}", display_file_name(&path));
    autosave_tui_current_workspace(workspace, runtime);
}

pub(crate) fn next_tui_untitled_path(
    workspace: &EditorWorkspace<'_>,
    runtime: &EditorRuntime,
) -> PathBuf {
    let directory = runtime
        .sidebar
        .as_ref()
        .map(|sidebar| sidebar.current_dir.clone())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    for index in 1.. {
        let file_name = if index == 1 {
            "untitled.txt".to_string()
        } else {
            format!("untitled-{index}.txt")
        };
        let candidate = directory.join(file_name);
        let already_open = workspace
            .tabs
            .iter()
            .any(|tab| tab.document.as_ref().path == candidate);
        if !already_open && !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("untitled candidate search is unbounded")
}
