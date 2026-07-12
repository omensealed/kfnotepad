//! Application and syntax theme cycling.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn cycle_theme(&mut self) {
        self.settings.theme_id = self.settings.theme_id.next();
        self.status_message = format!("theme: {}", self.settings.theme_id.label());
        self.persist_settings();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
    }

    pub(in crate::gui) fn cycle_syntax_theme(&mut self) {
        self.settings.syntax_theme_id = self.settings.syntax_theme_id.next();
        self.status_message = format!("syntax theme: {}", self.settings.syntax_theme_id.label());
        self.persist_settings();
        self.invalidate_all_syntax_caches();
        self.refresh_visible_syntax_caches();
    }
}
