impl KfnotepadGui {
    pub(super) fn remember_search_query(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        self.search_history.retain(|entry| entry != query);
        self.search_history.insert(0, query.to_string());
        self.search_history.truncate(GUI_FIND_HISTORY_LIMIT);
    }

    pub(super) fn select_search_history(&mut self, query: String) {
        self.search_query = query;
        self.search_history_open = false;
        self.search_active(false);
    }
}
