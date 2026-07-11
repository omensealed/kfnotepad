impl KfnotepadGui {
    pub(super) fn set_restore_last_workspace(&mut self, enabled: bool) {
        let message = if enabled {
            "restore last workspace: on".to_string()
        } else {
            "restore last workspace: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.gui_restore_last_workspace = enabled,
            message,
        );
    }

    pub(super) fn set_show_line_numbers(&mut self, enabled: bool) {
        let message = if enabled {
            "line numbers: on".to_string()
        } else {
            "line numbers: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.show_line_numbers = enabled,
            message,
        );
    }

    pub(super) fn set_wrap_lines(&mut self, enabled: bool) {
        let message = if enabled {
            "wrap text: on".to_string()
        } else {
            "wrap text: off".to_string()
        };
        self.update_settings_with_rollback(|settings| settings.wrap_lines = enabled, message);
    }

    pub(super) fn set_search_case_sensitive(&mut self, enabled: bool) {
        let message = if enabled {
            "search case sensitive: on".to_string()
        } else {
            "search case sensitive: off".to_string()
        };
        self.update_settings_with_rollback(
            |settings| settings.search_case_sensitive = enabled,
            message,
        );
        self.search_highlight = None;
    }
}
