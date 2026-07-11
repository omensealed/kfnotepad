//! Runtime construction and transient-state reset behavior.

use super::*;

impl Default for EditorRuntime {
    fn default() -> Self {
        Self {
            status: String::from("Ctrl-S save | Ctrl-Q quit"),
            quit_confirmation_pending: false,
            close_tab_confirmation_pending: false,
            search_active: false,
            search_query: String::new(),
            last_search_query: String::new(),
            search_history: Vec::new(),
            search_history_index: None,
            goto_line_active: false,
            goto_line_query: String::new(),
            menu: None,
            page_rows: 20,
            settings: EditorSettings::default(),
            config_path: None,
            workspace_projects_dir: None,
            workspace_prompt: None,
            workspace_query: String::new(),
            workspace_pending_open: None,
            workspace_pending_delete: None,
            workspace_prompt_candidates: Vec::new(),
            workspace_prompt_candidate_index: None,
            workspace_open_confirmation_pending: false,
            workspace_manager: None,
            sidebar: None,
            last_sidebar_dir: None,
            sidebar_prompt: None,
            sidebar_query: String::new(),
            overwrite_mode: false,
            reader_scroll_milli_lines: 0,
            command_palette: None,
        }
    }
}

impl EditorRuntime {
    pub(crate) fn search_highlight(&self) -> Option<SearchHighlightView<'_>> {
        if self.last_search_query.is_empty() {
            return None;
        }
        Some(SearchHighlightView {
            query: &self.last_search_query,
            mode: current_search_mode(self),
        })
    }
}
