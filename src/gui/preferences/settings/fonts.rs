//! Editor/UI font family and size preferences.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn cycle_gui_font_family(&mut self) {
        let next = self.settings.gui_font_family.next();
        self.update_settings_with_rollback(
            |settings| settings.gui_font_family = next,
            format!("font: {}", next.display_label()),
        );
    }

    pub(in crate::gui) fn set_gui_font_size(&mut self, size: u16) {
        if !(MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&size) {
            self.status_message =
                format!("editor font size must be {MIN_GUI_FONT_SIZE}-{MAX_GUI_FONT_SIZE}");
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_font_size = size,
            format!("editor font size: {size}"),
        );
    }

    pub(in crate::gui) fn set_gui_ui_font_size(&mut self, size: u16) {
        if !(MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE).contains(&size) {
            self.status_message =
                format!("ui font size must be {MIN_GUI_FONT_SIZE}-{MAX_GUI_FONT_SIZE}");
            return;
        }
        self.update_settings_with_rollback(
            |settings| settings.gui_ui_font_size = size,
            format!("ui font size: {size}"),
        );
    }
}
