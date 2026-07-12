//! Reader-mode enablement and speed preferences.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn set_reader_mode_enabled(&mut self, enabled: bool) {
        let message = if enabled {
            "reader mode: on".to_string()
        } else {
            "reader mode: off".to_string()
        };
        self.reader_scroll_accumulator = 0.0;
        self.update_settings_with_rollback(
            |settings| settings.gui_reader_mode_enabled = enabled,
            message,
        );
    }

    pub(in crate::gui) fn toggle_reader_mode(&mut self) {
        self.set_reader_mode_enabled(!self.settings.gui_reader_mode_enabled);
    }

    pub(in crate::gui) fn set_reader_speed(&mut self, lines_per_minute: u16) {
        if !(MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE)
            .contains(&lines_per_minute)
        {
            self.status_message = format!(
                "reader speed must be {MIN_GUI_READER_LINES_PER_MINUTE}-{MAX_GUI_READER_LINES_PER_MINUTE} lines/min"
            );
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_reader_lines_per_minute = lines_per_minute,
            format!("reader speed: {lines_per_minute} lines/min"),
        );
    }
}
