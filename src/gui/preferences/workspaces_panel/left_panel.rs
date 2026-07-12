//! Left-panel visibility and mode selection.

use super::*;

impl KfnotepadGui {
    pub(in crate::gui) fn toggle_left_panel(&mut self) {
        self.left_panel.toggle_visibility();
        self.browser_visible = self.left_panel.visible;
        self.status_message = if self.left_panel.visible {
            format!(
                "{} panel shown",
                self.left_panel.title().to_ascii_lowercase()
            )
        } else {
            format!(
                "{} panel hidden",
                self.left_panel.title().to_ascii_lowercase()
            )
        };
        self.persist_layout();
    }
    pub(in crate::gui) fn select_left_panel_mode(&mut self, mode: GuiLeftPanelMode) {
        match mode {
            GuiLeftPanelMode::Files => self.left_panel.show_files(),
            GuiLeftPanelMode::Workspaces => self.left_panel.show_workspaces(),
            GuiLeftPanelMode::Preferences => self.left_panel.show_preferences(),
        }
        self.browser_visible = self.left_panel.visible;
        self.status_message = format!(
            "{} panel shown",
            self.left_panel.title().to_ascii_lowercase()
        );
        self.persist_layout();
    }
}
