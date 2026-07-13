//! Test-only direct browser selection and activation helpers.

use super::*;

impl KfnotepadGui {
    #[cfg(test)]
    pub(in crate::gui::app::state) fn activate_browser_entry(&mut self, index: usize) {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return;
        }
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        if index >= browser.sidebar.entries.len() {
            return;
        }

        browser.sidebar.selected = index;
        if let Some(entry) = browser.sidebar.entries.get(index) {
            self.browser_selected_path = Some(entry.path.clone());
        }
        self.open_selected_browser_entry();
    }

    #[cfg(test)]
    pub(in crate::gui::app::state) fn select_browser_entry(&mut self, index: usize) {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return;
        }
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        if let Some(entry) = browser.sidebar.entries.get(index) {
            self.status_message = format!("selected {}", entry.path.display());
            browser.sidebar.selected = index;
            self.browser_selected_path = Some(entry.path.clone());
            self.pending_browser_delete = None;
        }
    }

    #[cfg(test)]
    pub(in crate::gui::app::state) fn open_selected_browser_entry(&mut self) {
        let Some(browser) = self.browser.as_mut() else {
            self.status_message = "file browser unavailable".to_string();
            return;
        };
        match browser.activate_selected() {
            Ok(kfnotepad::GuiFileBrowserActivation::Navigated { current_dir }) => {
                self.status_message = format!("browser: {}", current_dir.display());
            }
            Ok(kfnotepad::GuiFileBrowserActivation::OpenTile { path }) => {
                let _opened = self.open_path_in_new_pane(path);
            }
            Err(error) => {
                self.status_message = format!("file browser error: {error}");
            }
        }
    }
}
