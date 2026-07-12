//! Settings mutation with atomic rollback on persistence failure.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn update_settings_with_rollback(
        &mut self,
        update: impl FnOnce(&mut EditorSettings),
        success_message: impl Into<String>,
    ) {
        let previous = self.settings;
        update(&mut self.settings);
        if let Some(config_path) = self.config_path.as_deref() {
            if let Err(error) = save_editor_settings(config_path, self.settings) {
                self.settings = previous;
                self.status_message = format!("settings save failed: {error}");
                return;
            }
        }
        self.status_message = success_message.into();
    }
}
