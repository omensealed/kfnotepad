//! Built-in help-document tab activation.

use super::*;

pub(crate) fn open_tui_help_document(
    workspace: &mut EditorWorkspace<'_>,
    runtime: &mut EditorRuntime,
) {
    let help_path = PathBuf::from(TUI_HELP_DOCUMENT_PATH);
    if let Some(index) = workspace
        .tabs
        .iter()
        .position(|tab| tab.document.as_ref().path == help_path)
    {
        workspace.active = index;
        runtime.menu = None;
        runtime.search_active = false;
        runtime.goto_line_active = false;
        stop_reader_mode(runtime, "Reader mode stopped for help");
        runtime.status = String::from("Focused help");
        return;
    }

    let document = TextDocument {
        path: help_path,
        buffer: kfnotepad::TextBuffer::from_text(tui_help_document_text()),
    };
    workspace.push_owned_tab(document);
    runtime.menu = None;
    runtime.search_active = false;
    runtime.goto_line_active = false;
    runtime.quit_confirmation_pending = false;
    runtime.close_tab_confirmation_pending = false;
    stop_reader_mode(runtime, "Reader mode stopped for help");
    runtime.status = String::from("Opened help");
}
