//! Search, navigation, editor, reader, theme, and menu messages.

use super::*;

pub(super) fn dispatch_search_and_editor(
    state: &mut KfnotepadGui,
    message: Message,
) -> GuiDispatchResult {
    match message {
        Message::Edit(pane, action) => {
            GuiDispatchResult::Handled(handle_editor_edit(state, pane, action))
        }
        Message::ReaderScrollTick => {
            state.reader_scroll_tick();
            handled_none()
        }
        Message::CycleTheme => {
            state.cycle_theme();
            handled_none()
        }
        Message::CycleSyntaxTheme => {
            state.cycle_syntax_theme();
            handled_none()
        }
        Message::SearchQueryChanged(query) => {
            handle_search_query_changed(state, query);
            handled_none()
        }
        Message::SearchHistorySelected(query) => {
            state.select_search_history(query);
            handled_none()
        }
        Message::GoToLineQueryChanged(query) => {
            handle_go_to_line_query_changed(state, query);
            handled_none()
        }
        Message::SearchNext => {
            state.search_active(false);
            handled_none()
        }
        Message::SearchPrevious => {
            state.search_active(true);
            handled_none()
        }
        Message::GoDocumentStart => {
            handle_go_document_start(state);
            handled_none()
        }
        Message::GoDocumentEnd => {
            handle_go_document_end(state);
            handled_none()
        }
        Message::GoToLineRequested => {
            handle_go_to_line_requested(state);
            handled_none()
        }
        Message::ScrollActiveEditorViewport(delta) => {
            state.scroll_active_editor_viewport(delta);
            handled_none()
        }
        Message::MenuCommand(command) => {
            GuiDispatchResult::Handled(state.run_menu_command(command))
        }
        Message::ClipboardPasted(contents) => {
            state.paste_into_active_editor(contents);
            handled_none()
        }
        other => GuiDispatchResult::Unhandled(other),
    }
}
