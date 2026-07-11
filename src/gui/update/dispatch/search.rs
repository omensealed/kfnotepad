fn handle_search_query_changed(state: &mut KfnotepadGui, query: String) {
    state.search_query = query;
    state.search_history_open = state.search_query.is_empty() && !state.search_history.is_empty();
    state.search_highlight = None;
}

fn handle_go_to_line_query_changed(state: &mut KfnotepadGui, query: String) {
    state.go_to_line_query = query;
}

fn handle_go_document_start(state: &mut KfnotepadGui) {
    state.search_highlight = None;
    state.go_active_document_start();
}

fn handle_go_document_end(state: &mut KfnotepadGui) {
    state.search_highlight = None;
    state.go_active_document_end();
}

fn handle_go_to_line_requested(state: &mut KfnotepadGui) {
    state.search_highlight = None;
    state.go_active_line();
}
