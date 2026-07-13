//! Frequently used active-editor and browser-directory accessors.

use super::*;

impl KfnotepadGui {
    #[cfg(test)]
    pub(in crate::gui::app::state) fn active_editor(&self) -> &GuiEditorAdapter {
        &self
            .panes
            .get(self.active_pane)
            .expect("active GUI pane must exist")
            .editor
    }

    pub(in crate::gui::app::state) fn current_browser_dir(&self) -> PathBuf {
        self.browser
            .as_ref()
            .map(|browser| browser.sidebar.current_dir.clone())
            .unwrap_or_else(|| self.current_dir.clone())
    }
}
